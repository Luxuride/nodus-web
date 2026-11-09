//! The `:draft` pickup list.
//!
//! Typing `:draft` is a request for what was left unsent, in the same place the emoji
//! suggestions appear — one popup vocabulary for everything the composer can insert. The
//! store is the caller's, because `localStorage` is the browser's: it hands this list over
//! already read and already aged.

use leptos::prelude::*;

/// One unsent draft, as the popup shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Draft {
    /// The first lines of the stored text.
    pub label: String,
    /// The visibility it was saved with.
    pub visibility: String,
    /// The language's name, or "instance default" when none was chosen.
    pub language: String,
    /// How long ago it was written, "3 days ago".
    pub age: String,
}

/// The open `:draft` popup.
#[component]
pub fn DraftsPanel(
    /// True while the writer's text ends in `:draft`.
    #[prop(into)]
    open: Signal<bool>,
    #[prop(into)] drafts: Signal<Vec<Draft>>,
    /// Called with the index of the chosen draft.
    #[prop(optional)]
    insert: Option<Callback<usize>>,
) -> impl IntoView {
    view! {
        <Show when=move || open.get() && !drafts.get().is_empty()>
            <div
                class="max-h-64 overflow-y-auto rounded-box border border-base-300 bg-base-100 p-1 shadow-lg"
                role="listbox"
                data-drafts=""
                aria-label="Drafts"
            >
                {move || drafts.get().into_iter().enumerate().map(|(index, draft)| {
                    let label = draft.label;
                    let visibility = draft.visibility;
                    let language = draft.language;
                    let age = draft.age;
                    view! {
                        <button
                            type="button"
                            role="option"
                            aria-selected="false"
                            class="flex w-full flex-col items-start gap-0.5 rounded-lg px-2 py-1 text-left hover:bg-base-200"
                            on:click=move |event| {
                                #[cfg(target_arch = "wasm32")]
                                event.prevent_default();
                                #[cfg(not(target_arch = "wasm32"))]
                                let _ = &event;
                                if let Some(insert) = insert {
                                    insert.run(index);
                                }
                            }
                        >
                            <span class="line-clamp-2 text-sm">{label}</span>
                            <span class="text-xs opacity-60">
                                {visibility}
                                " · "
                                {language}
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
