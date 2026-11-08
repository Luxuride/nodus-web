//! The `:draft` pickup list.
//!
//! Typing `:draft` is a request for what was left unsent, in the same place the emoji
//! suggestions appear — one popup vocabulary for everything the composer can insert. A
//! choice fills the composer with that draft's text, visibility and language, and removes
//! it from the list, because a draft that stays put after being resumed is a draft that
//! gets posted twice.

use leptos::prelude::*;

use crate::ui::drafts::{self, Saved};

/// The open `:draft` popup.
#[component]
pub(super) fn DraftsPanel(
    /// True while the writer's text ends in `:draft`.
    open: Signal<bool>,
    /// Called with the index of the chosen draft.
    insert: Callback<usize>,
) -> impl IntoView {
    let (drafts, set_drafts) = signal(Vec::<Saved>::new());

    // Reload when the popup opens rather than keeping a second copy of `localStorage` in
    // memory: the store changes in another tab as easily as in this one.
    Effect::new(move |_| {
        if open.get() {
            set_drafts.set(drafts::all());
        }
    });

    view! {
        <Show when=move || open.get() && !drafts.get().is_empty()>
            <div
                class="max-h-64 overflow-y-auto rounded-box border border-base-300 bg-base-100 p-1 shadow-lg"
                role="listbox"
                aria-label="Drafts"
            >
                {move || drafts.get().into_iter().enumerate().map(|(index, draft)| {
                    // `to_iso_string` hands back a `JsString`, and `relative_time` wants the
                    // `String` this crate parses dates from.
                    let when = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(draft.at));
                    let age = crate::routes::format::relative_time(&String::from(when.to_iso_string()));
                    let label = drafts::summary(&draft.text);
                    let language = draft.language.clone();
                    view! {
                        <button
                            type="button"
                            role="option"
                            aria-selected="false"
                            class="flex w-full flex-col items-start gap-0.5 rounded-lg px-2 py-1 text-left hover:bg-base-200"
                            on:click=move |event| {
                                event.prevent_default();
                                insert.run(index);
                            }
                        >
                            <span class="line-clamp-2 text-sm">{label}</span>
                            <span class="text-xs opacity-60">
                                {draft.visibility.clone()}
                                " · "
                                {if language.is_empty() { "instance default".to_owned() } else { language }}
                                " · "
                                {age}
                            </span>
                        </button>
                    }
                }).collect_view()}
            </div>
        </Show>
    }
}
