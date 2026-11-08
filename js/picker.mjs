//! The emoji picker, loaded the first time a composer asks for one.
//!
//! Kept out of `editor.js` on purpose: the picker carries an emoji database an order of
//! magnitude larger than the editor, and a reader who never opens it should never download
//! it. `load()` is idempotent, so two composers opening at once share one module.

import "emoji-picker-element";

let ready = null;

function create(target, customEmoji = [], onPick = () => {}) {
  target.replaceChildren();

  const picker = document.createElement("emoji-picker");
  // The database is served from our own origin (`vendor/emoji-data.json`, copied from the
  // package at build time): a reader's client should not tell a CDN what someone is typing.
  picker.dataSource = "/vendor/emoji-data.json";
  picker.classList.add("rounded-box", "border", "border-base-300", "bg-base-100");
  // `emoji-picker-element` sorts the array it is handed, in place, so it gets a copy: the
  // Rust side's catalogue must not be reordered by opening the picker.
  picker.customEmoji = customEmoji.map((emoji) => ({
    name: emoji.shortcode,
    shortcodes: [emoji.shortcode],
    url: emoji.url || emoji.static_url || "",
  }));
  picker.addEventListener("emoji-click", (event) => {
    const detail = event.detail ?? {};
    const unicode = detail.unicode ?? detail.emoji?.unicode;
    if (unicode) {
      onPick(unicode);
      return;
    }
    const shortcode =
      detail.emoji?.shortcodes?.find((entry) => entry.trim().length > 0) ??
      detail.name ??
      detail.emoji?.name;
    if (shortcode) {
      onPick(`:${String(shortcode).trim().replace(/^:+|:+$/g, "")}:`);
    }
  });

  target.append(picker);
  return picker;
}

window.nodusPicker = {
  load() {
    ready ??= Promise.resolve();
    return ready;
  },
  create,
};
