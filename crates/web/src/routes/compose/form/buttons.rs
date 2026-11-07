//! The submit row: one button whose label is the only thing that changes between a new
//! status and an edit.

use leptos::prelude::*;

use crate::router::{self, Route};

/// Post or save, plus cancel.
#[component]
pub fn ComposerActions(
    text: ReadSignal<String>,
    busy: ReadSignal<bool>,
    editing: StoredValue<Option<String>>,
    on_submit: Callback<()>,
) -> impl IntoView {
    view! {
        <div class="flex items-center gap-2">
            <button
                class="btn btn-primary btn-sm"
                disabled=move || busy.get() || text.get().trim().is_empty()
                on:click=move |_| on_submit.run(())
            >
                {move || if busy.get() {
                    "Sending…".to_owned()
                } else if editing.get_value().is_some() {
                    "Save changes".to_owned()
                } else {
                    "Post".to_owned()
                }}
            </button>
            <button class="btn btn-ghost btn-sm" on:click=move |_| router::navigate(Route::Home)>
                "Cancel"
            </button>
        </div>
    }
}
