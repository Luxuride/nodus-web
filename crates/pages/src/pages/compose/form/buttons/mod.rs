//! The submit row: one button whose label is the only thing that changes between a new
//! status and an edit.
//!
//! The row reads no state of its own: the caller says whether a send is in flight, whether
//! the composer is editing, and what the two presses do. A caller with no verb still gets
//! the same row, with the buttons disabled rather than missing.

use leptos::prelude::*;

/// Post or save, plus cancel.
#[component]
pub fn ComposerActions(
    /// The draft as typed; a blank body keeps the send button disabled.
    #[prop(into)]
    text: Signal<String>,
    /// Whether a send is already on its way.
    #[prop(into)]
    busy: Signal<bool>,
    /// Whether this composer edits a post that already exists, which renames the button.
    #[prop(optional)]
    editing: bool,
    /// The character limit, when a longer body becomes a thread.
    #[prop(optional)]
    limit: Option<usize>,
    /// Sends the post.
    #[prop(optional)]
    on_submit: Option<Callback<()>>,
    /// Leaves the composer.
    #[prop(optional)]
    on_cancel: Option<Callback<()>>,
) -> impl IntoView {
    view! {
        <div class="flex items-center gap-2">
            <Show when=move || !editing && limit.is_some_and(|limit| crate::pages::compose::thread::posts(&text.get(), limit) > 1)>
                <p class="text-xs opacity-60">
                    {move || {
                        let posts = limit.map_or(1, |limit| {
                            crate::pages::compose::thread::posts(&text.get(), limit)
                        });
                        format!("This will go out as a thread of {posts} posts.")
                    }}
                </p>
            </Show>
            <button
                class="btn btn-primary btn-sm"
                disabled=move || busy.get() || text.get().trim().is_empty() || on_submit.is_none()
                on:click=move |_| {
                    if let Some(on_submit) = on_submit {
                        on_submit.run(());
                    }
                }
            >
                {move || if busy.get() {
                    "Sending…".to_owned()
                } else if editing {
                    "Save changes".to_owned()
                } else if limit.is_some_and(|limit| crate::pages::compose::thread::posts(&text.get(), limit) > 1) {
                    "Post thread".to_owned()
                } else {
                    "Post".to_owned()
                }}
            </button>
            <button
                class="btn btn-ghost btn-sm"
                disabled=move || on_cancel.is_none()
                on:click=move |_| {
                    if let Some(on_cancel) = on_cancel {
                        on_cancel.run(());
                    }
                }
            >
                "Cancel"
            </button>
        </div>
    }
}
