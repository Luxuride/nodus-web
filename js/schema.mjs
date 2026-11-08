//! The post as a ProseMirror document.
//!
//! Mastodon stores plain text and returns HTML, so the editor's job is narrower than a chat
//! client's: one atom (a custom emoji) inside paragraphs of text. Everything else the wire
//! cannot carry — bold, links, headings — is deliberately absent from the schema, because an
//! editor that can express more than the API accepts lies about what will be posted.

import { Schema } from "prosemirror-model";

export const TOKEN = /:([A-Za-z0-9_+-]+):/g;

export const schema = new Schema({
  nodes: {
    doc: { content: "paragraph+" },
    paragraph: {
      content: "(text|customEmoji)*",
      parseDOM: [{ tag: "p" }],
      // The class list matches what the reader's body uses, so a post does not change
      // appearance between the composer and the timeline.
      toDOM: () => ["p", { class: "whitespace-pre-line text-sm leading-relaxed" }, 0],
    },
    text: {},
    customEmoji: {
      inline: true,
      group: "inline",
      atom: true,
      selectable: true,
      draggable: false,
      attrs: { token: {}, url: {} },
      parseDOM: [
        {
          tag: "img[data-custom-emoji-token]",
          getAttrs: (dom) => {
            if (!(dom instanceof HTMLImageElement)) {
              return false;
            }
            const token = dom.dataset.customEmojiToken?.trim().toLowerCase();
            if (!token || !/^:[a-z0-9_+-]+:$/.test(token)) {
              return false;
            }
            return { token, url: dom.getAttribute("src") ?? "" };
          },
        },
      ],
      toDOM: (node) => [
        "img",
        {
          src: node.attrs.url,
          alt: node.attrs.token,
          title: node.attrs.token,
          "data-custom-emoji-token": node.attrs.token,
          contenteditable: "false",
          class: "emoji inline-block h-[1.2em] w-auto align-[-0.2em]",
        },
      ],
      // Copying an emoji out of the editor yields its shortcode, which is what the API
      // wants back — not an invisible gap where an image used to be.
      leafText: (node) => node.attrs.token,
    },
  },
});

/// Everything the composer knows about the instance's emoji, keyed by `:token:`.
export function buildIndex(customEmoji) {
  const index = new Map();
  for (const emoji of customEmoji ?? []) {
    const token = `:${String(emoji.shortcode ?? "").toLowerCase()}:`;
    if (token.length > 2 && !index.has(token)) {
      index.set(token, emoji.url || emoji.static_url || "");
    }
  }
  return index;
}

/// The document for a draft, with known shortcodes already drawn as images.
export function fromDraft(draft, index) {
  const lines = String(draft ?? "").split("\n");
  const paragraphs = lines.map((line) => {
    const nodes = [];
    let last = 0;
    for (const match of line.matchAll(TOKEN)) {
      const start = match.index ?? 0;
      if (start > last) {
        nodes.push(schema.text(line.slice(last, start)));
      }
      const token = match[0].toLowerCase();
      const url = index.get(token);
      if (url) {
        nodes.push(schema.node("customEmoji", { token, url }));
      } else {
        // An unknown shortcode stays text: the instance would not render it either, and
        // turning it into an image would promise a picture the post cannot have.
        nodes.push(schema.text(match[0]));
      }
      last = start + match[0].length;
    }
    if (last < line.length) {
      nodes.push(schema.text(line.slice(last)));
    }
    return schema.node("paragraph", null, nodes);
  });
  if (paragraphs.length === 0) {
    paragraphs.push(schema.node("paragraph"));
  }
  return schema.node("doc", null, paragraphs);
}

/// The plain text the API takes, with emoji spelled as their shortcodes.
export function toDraft(doc) {
  const paragraphs = [];
  doc.forEach((node) => {
    paragraphs.push(
      node.textBetween(0, node.content.size, "\n", (leaf) =>
        leaf.type.name === "customEmoji" ? leaf.attrs.token : "",
      ),
    );
  });
  return paragraphs.join("\n");
}

/// Character count as the instance counts it: an emoji costs its shortcode, which is what
/// the server will store.
export function count(doc) {
  return toDraft(doc).length;
}
