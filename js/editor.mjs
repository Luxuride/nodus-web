//! The editor the Rust composer mounts.
//!
//! Contract, kept deliberately small because it is a wasm↔JS boundary:
//!   mount(element, options) → { getText, setText, insert, focus, count, destroy }
//!   options: { initial, placeholder, limit, emoji: [{shortcode, url}], onText(text), onSubmit() }
//!
//! `onText` is called on every document change and on nothing else, so the Rust side holds
//! one plain string per keystroke rather than a mirror of the document.

import { baseKeymap } from "prosemirror-commands";
import { history, redo, undo } from "prosemirror-history";
import { keymap } from "prosemirror-keymap";
import { EditorState, Plugin } from "prosemirror-state";
import { Decoration, DecorationSet, EditorView } from "prosemirror-view";

import { buildIndex, fromDraft, schema, toDraft } from "./schema.mjs";

const EMPTY = "Type your post…";

function placeholder(text) {
  return new Plugin({
    props: {
      // `decorations` is handed the EditorState, not the view: reading `view.state` here
      // threw on the first mount, which is the sort of mistake a probe catches in a second.
      decorations(state) {
        const { doc } = state;
        const empty = doc.childCount === 1 && doc.firstChild?.content.size === 0;
        if (!empty) {
          return null;
        }
        const widget = document.createElement("span");
        widget.className = "pointer-events-none absolute select-none opacity-50";
        widget.textContent = text;
        return DecorationSet.create(doc, [
          Decoration.widget(0, widget, { side: -1, ignoreSelection: true }),
        ]);
      },
    },
  });
}

function mount(element, options = {}) {
  const {
    initial = "",
    placeholder: placeholderText = EMPTY,
    emoji = [],
    onText = () => {},
    onSubmit = () => {},
    onPickEmoji = () => {},
    onQuery = () => {},
  } = options;

  if (typeof element.__nodusDestroy === "function") {
    element.__nodusDestroy();
  }

  let index = buildIndex(emoji);
  // Standard emoji, loaded once per document: the instance catalogue is often empty (a new
  // server publishes `[]`), and a composer that suggests nothing looks broken.
  let standard = new Map();
  const loadStandard = async () => {
    try {
      const response = await fetch("/vendor/emoji-data.json");
      const data = await response.json();
      for (const entry of data) {
        const char = entry.unicode;
        const code = entry.shortcodes?.[0] ?? entry.label?.toLowerCase().replace(/[^a-z0-9_]+/g, "_");
        if (char && code && !standard.has(code)) {
          standard.set(code, char);
        }
      }
    } catch (error) {
      console.warn("nodus: the standard emoji list did not load", error);
    }
  };
  void loadStandard();
  let codes = [...index.keys()].map((token) => token.slice(1, -1));
  // The token being completed right now, if any: `{ code, from, to }` in document positions.
  let hint = null;

  const matches = (code) => {
    // The instance's own emoji first — they are what a reader on this server expects — and
    // the standard set after it, so `:smile` still completes on an instance with none.
    const custom = codes.filter((candidate) => candidate.startsWith(code));
    const rest = codes.filter((c) => !c.startsWith(code) && c.includes(code));
    const plain = [...standard.keys()].filter((candidate) => candidate.startsWith(code));
    const found = [...custom, ...rest, ...plain].filter(
      (candidate, at, all) => all.indexOf(candidate) === at,
    );
    return found.slice(0, 8);
  };

  /** The `:token` immediately before the caret, with the range it occupies. */
  const trailing = (state) => {
    const { $from } = state.selection;
    if (!$from.parent.isTextblock) {
      return null;
    }
    const before = $from.parent.textBetween(0, $from.parentOffset, undefined, "\ufffc");
    const found = /:([A-Za-z0-9_+-]{1,40})$/.exec(before);
    if (!found) {
      return null;
    }
    const code = found[1];
    const options = matches(code);
    if (options.length === 0) {
      return null;
    }
    return { code, from: $from.pos - code.length - 1, to: $from.pos, options };
  };

  /** Replaces the pending token with the emoji the instance knows. */
  const accept = (code) => {
    const token = `:${code}:`;
    const url = index.get(token);
    if (!url && standard.has(code)) {
      // A standard emoji is inserted as its character: an image atom for it would be wrong
      // on other software, and unreadable in plain text.
      view.dispatch(view.state.tr.insertText(standard.get(code)).scrollIntoView());
      view.focus();
      return true;
    }
    const range = hint ?? trailing(view.state);
    if (!url || !range) {
      return false;
    }
    const node = schema.node("customEmoji", { token, url });
    const tr = view.state.tr.replaceWith(range.from, range.to, node).scrollIntoView();
    view.dispatch(tr);
    view.focus();
    return true;
  };

  const state = EditorState.create({
    schema,
    doc: fromDraft(initial, index),
    plugins: [
      history(),
      keymap({ "Mod-z": undo, "Shift-Mod-z": redo, "Mod-y": redo }),
      keymap(baseKeymap),
      placeholder(placeholderText),
    ],
  });

  const view = new EditorView(element, {
    state,
    attributes: {
      class:
        "min-h-24 w-full whitespace-pre-line text-sm leading-relaxed outline-none",
      "aria-label": "Post body",
      role: "textbox",
      "aria-multiline": "true",
    },
    // Enter posts and Shift-Enter breaks the line: the same pair the textarea had, so a
    // habit formed there keeps working.
    handleKeyDown: (view, event) => {
      if (event.key === "Escape" && hint) {
        hint = null;
        onQuery("", []);
        return true;
      }
      if (event.key === "Tab" && hint) {
        event.preventDefault();
        accept(hint.options[0]);
        return true;
      }
      if (event.key !== "Enter" || event.isComposing) {
        return false;
      }
      // While a token is being completed, Enter takes the suggestion: posting a half-typed
      // `:blob` is never what the writer meant.
      if (hint) {
        event.preventDefault();
        accept(hint.options[0]);
        return true;
      }
      if (event.shiftKey) {
        return false;
      }
      event.preventDefault();
      onSubmit();
      return true;
    },
    dispatchTransaction(transaction) {
      const next = view.state.apply(transaction);
      view.updateState(next);
      // A finished `:token:` becomes an atom as soon as it is typed, so the writer sees the
      // picture they will publish rather than the code the instance happens to store.
      if (transaction.docChanged) {
        const resolved = resolveTrailingToken(next, view.state.doc);
        if (resolved) {
          view.dispatch(resolved);
        }
      }
      if (transaction.docChanged || transaction.selectionSet) {
        const before = hint;
        hint = trailing(view.state);
        if (hint?.code !== before?.code || hint?.options.length !== before?.options.length) {
          onQuery(hint?.code ?? "", hint?.options ?? []);
        }
        onText(toDraft(view.state.doc));
      }
    },
  });

  const insert = (text) => {
    if (!text) {
      return;
    }
    const { from, to } = view.state.selection;
    // An emoji arrives as `:token:`; it becomes an atom when the instance knows the code,
    // and stays text when it does not — the same rule the reader applies.
    if (/^:[A-Za-z0-9_+-]+:$/.test(text) && index.has(text)) {
      const node = schema.node("customEmoji", { token: text, url: index.get(text) });
      view.dispatch(view.state.tr.replaceSelectionWith(node).scrollIntoView());
    } else {
      view.dispatch(view.state.tr.insertText(text, from, to).scrollIntoView());
    }
    view.focus();
  };

  /** The transaction that turns a just-completed `:token:` into its atom, if there is one. */
  const resolveTrailingToken = (state, previous) => {
    const { $from, empty } = state.selection;
    if (!empty || !$from.parent.isTextblock) {
      return null;
    }
    const before = $from.parent.textBetween(0, $from.parentOffset, undefined, "\ufffc");
    const found = /:([A-Za-z0-9_+-]+):$/.exec(before);
    if (!found) {
      return null;
    }
    const token = found[0];
    const url = index.get(token);
    if (!url) {
      return null;
    }
    const from = $from.pos - token.length;
    const node = schema.node("customEmoji", { token, url });
    void previous;
    return state.tr.replaceWith(from, $from.pos, node).scrollIntoView();
  };

  const destroy = () => {
    view.destroy();
    delete element.__nodusDestroy;
  };
  element.__nodusDestroy = destroy;

  view.focus();

  return {
    getText: () => toDraft(view.state.doc),
    count: () => toDraft(view.state.doc).length,
    setText: (text) => {
      // Only called when the draft changed from outside (an edit loaded, a cleared form);
      // rebuilding is cheaper than diffing a document the UI did not see.
      const next = EditorState.create({
        schema,
        doc: fromDraft(text, index),
        plugins: view.state.plugins,
      });
      view.updateState(next);
      onText(toDraft(next.doc));
    },
    setEmoji: (list) => {
      index = buildIndex(list);
      codes = [...index.keys()].map((token) => token.slice(1, -1));
    },
    accept,
    insert,
    focus: () => view.focus(),
    destroy,
  };
}

window.nodusEditor = { mount };
