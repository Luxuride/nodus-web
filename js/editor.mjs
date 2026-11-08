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
    placeholder: hint = EMPTY,
    emoji = [],
    onText = () => {},
    onSubmit = () => {},
    onPickEmoji = () => {},
  } = options;

  if (typeof element.__nodusDestroy === "function") {
    element.__nodusDestroy();
  }

  let index = buildIndex(emoji);

  const state = EditorState.create({
    schema,
    doc: fromDraft(initial, index),
    plugins: [
      history(),
      keymap({ "Mod-z": undo, "Shift-Mod-z": redo, "Mod-y": redo }),
      keymap(baseKeymap),
      placeholder(hint),
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
      if (event.key !== "Enter" || event.isComposing) {
        return false;
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
      if (transaction.docChanged || transaction.selectionSet) {
        onText(toDraft(next.doc));
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
    },
    insert,
    focus: () => view.focus(),
    destroy,
  };
}

window.nodusEditor = { mount };
