//! The emoji hints above the writing box: the codes a typed `:token` matches.
//!
//! The editor decides which codes match and where the list belongs; this module draws the
//! rows and reports the one the writer clicked.

use leptos::prelude::*;

/// One row: the code, and what to draw for it (a URL or a character).
#[derive(Clone, Debug, PartialEq)]
pub struct EmojiHint {
    pub code: String,
    pub preview: String,
}

/// The hints the editor is offering, and where the caret is inside the editor box.
#[derive(Clone, Debug, PartialEq)]
pub struct EmojiHintsValue {
    pub options: Vec<EmojiHint>,
    pub left: f64,
    pub top: f64,
    /// Which row Enter and Tab will take; the arrows move it in the editor.
    pub active: usize,
}

/// The hint list, positioned over the caret.
#[component]
pub fn EmojiHints(
    #[prop(into)] value: Signal<Option<EmojiHintsValue>>,
    /// Takes the code of the row that was pressed.
    #[prop(optional_no_strip)]
    on_accept: Option<Callback<String>>,
) -> impl IntoView {
    view! {
        <Show when=move || value.get().is_some()>
            {move || {
                let hints = value.get()?;
                let style = format!(
                    "left: min({}px, calc(100% - 16rem)); top: {}px;",
                    hints.left, hints.top
                );
                Some(view! {
                    <ul
                        class="absolute z-40 max-h-48 w-64 overflow-y-auto rounded-box border border-base-300 bg-base-100 p-1 shadow-xl"
                        style=style
                        data-emoji-hints=""
                        role="listbox"
                        aria-label="Emoji matching the typed code"
                    >
                        {hints
                            .options
                            .into_iter()
                            .enumerate()
                            .map(|(row, hint)| {
                                let code = hint.code.clone();
                                let preview = hint.preview.clone();
                                let label = format!(":{code}:");
                                let face = if preview.starts_with("http") {
                                    view! { <img class="h-5 w-5 object-contain" src=preview alt=""/> }
                                        .into_any()
                                } else {
                                    view! { <span class="text-base">{preview}</span> }.into_any()
                                };
                                let picked = row == hints.active;
                                let tone = if picked {
                                    "flex w-full items-center gap-2 rounded bg-base-200 px-2 py-1 text-left text-sm"
                                } else {
                                    "flex w-full items-center gap-2 rounded px-2 py-1 text-left text-sm hover:bg-base-200"
                                };
                                view! {
                                    <li
                                        role="option"
                                        class="flex items-center gap-2"
                                        aria-selected=picked.to_string()
                                        data-active=picked.to_string()
                                    >
                                        <button
                                            type="button"
                                            class=tone
                                            on:mousedown=move |event| {
                                                event.prevent_default();
                                                if let Some(on_accept) = on_accept {
                                                    on_accept.run(code.clone());
                                                }
                                            }
                                        >
                                            {face}
                                            <span class="truncate">{label}</span>
                                        </button>
                                    </li>
                                }
                            })
                            .collect_view()}
                    </ul>
                })
            }}
        </Show>
    }
}
