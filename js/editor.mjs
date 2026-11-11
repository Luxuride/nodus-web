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
      // The data is emojibase's: the character is `emoji`, and every alias matters — `smile`
      // is the second shortcode of the grinning-face-with-smiling-eyes entry.
      for (const entry of data) {
        const char = entry.emoji;
        for (const code of entry.shortcodes ?? []) {
          if (char && code && !standard.has(code)) {
            standard.set(code, char);
          }
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
    // Each row carries what to draw: the instance's picture, or the character itself, so the
    // list shows emoji rather than a list of codes the writer has to decode.
    return found.slice(0, 8).map((candidate) => [
      candidate,
      index.get(`:${candidate}:`) ?? standard.get(candidate) ?? "",
    ]);
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
    // The first row is selected by default, so Enter and Tab work without looking.
    return { code, from: $from.pos - code.length - 1, to: $from.pos, options, active: 0 };
  };

  /** Replaces the pending token — the one typed, not the one chosen — with the emoji. */
  /**
   * Sends the pending token, its options, the caret and the highlighted row to Rust.
   *
   * Called on every keystroke and on every arrow press: the arrow changes which row is
   * highlighted, and the list re-renders from this, so there is one source of truth.
   */
  const emit = () => {
    if (!hint) {
      onQuery("", [], 0, 0, 0);
      return;
    }
    let left = 0;
    let top = 0;
    try {
      const caret = view.coordsAtPos(view.state.selection.from);
      const host = element.getBoundingClientRect();
      left = Math.max(0, Math.round(caret.left - host.left));
      top = Math.round(caret.bottom - host.top);
    } catch {
      left = 0;
      top = 0;
    }
    onQuery(hint.code, hint.options, left, top, hint.active);
  };

  /** Which suggestion Enter and Tab will take. */
  const chosen = () => hint && hint.options[hint.active] ? hint.options[hint.active][0] : null;

  const accept = (code) => {
    if (!code) {
      return false;
    }
    // The range comes from what is actually in the document: `:smi` may accept `:smiley:`,
    // and replacing `code.length` characters from the caret would eat text before the token.
    const range = trailing(view.state) ?? hint;
    const url = index.get(`:${code}:`);
    const char = standard.get(code);
    if (!url && !char) {
      return false;
    }
    const from = range ? range.from : view.state.selection.from;
    const to = range ? range.to : view.state.selection.from;
    const node = url ? schema.node("customEmoji", { token: `:${code}:`, url }) : schema.text(char);
    view.dispatch(view.state.tr.replaceWith(from, to, node).scrollIntoView());
    hint = null;
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
        "min-h-24 w-full whitespace-pre-line break-words text-sm leading-relaxed outline-none [overflow-wrap:anywhere]",
      "aria-label": "Post body",
      role: "textbox",
      "aria-multiline": "true",
    },
    // Enter breaks the line like any multiline field. Posting is Mod-Enter, so a long
    // thought is never sent by the keystroke that was meant to start its next line.
    handleKeyDown: (view, event) => {
      if (event.key === "Escape" && hint) {
        hint = null;
        emit();
        return true;
      }
      if (hint && (event.key === "ArrowDown" || event.key === "ArrowUp")) {
        event.preventDefault();
        const step = event.key === "ArrowDown" ? 1 : -1;
        const last = hint.options.length - 1;
        hint.active = Math.min(last, Math.max(0, hint.active + step));
        emit();
        return true;
      }
      if (event.key === "Tab" && hint) {
        event.preventDefault();
        const picked = chosen();
        if (picked) {
          accept(picked);
        }
        return true;
      }
      if (event.key !== "Enter" || event.isComposing) {
        return false;
      }
      // While a token is being completed, Enter takes the suggestion: posting a half-typed
      // `:blob` is never what the writer meant.
      if (hint) {
        event.preventDefault();
        const picked = chosen();
        if (picked) {
          accept(picked);
        }
        return true;
      }
      if (!event.metaKey && !event.ctrlKey) {
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
          emit();
        }
      }
      if (transaction.docChanged) {
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
    const code = found[1];
    const url = index.get(token);
    const from = $from.pos - token.length;
    void previous;
    if (url) {
      const node = schema.node("customEmoji", { token, url });
      return state.tr.replaceWith(from, $from.pos, node).scrollIntoView();
    }
    // A standard emoji becomes its character the moment the code is closed, so `:smile:`
    // reads as 😄 while typing — the same promise the custom atom keeps.
    const char = standard.get(code);
    if (char) {
      return state.tr.replaceWith(from, $from.pos, schema.text(char)).scrollIntoView();
    }
    return null;
  };

  const destroy = () => {
    view.destroy();
    delete element.__nodusDestroy;
  };
  element.__nodusDestroy = destroy;

  // Mounted, not focused: a composer that grabs the caret on every page load moves the
  // reader off whatever they were doing (and on a phone it raises the keyboard).

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
