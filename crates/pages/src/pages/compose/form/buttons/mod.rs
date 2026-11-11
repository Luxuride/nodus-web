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
    /// The instance's character limit.
    #[prop(into)]
    limit: Signal<usize>,
    /// Extra pages the writer laid out by hand, beyond the one in the main editor.
    #[prop(optional_no_strip)]
    extra: Option<Signal<usize>>,
    /// Whether the visual thread editor is open.
    #[prop(into)]
    thread_mode: Signal<bool>,
    /// Whether a page is over the limit and has to be split first.
    #[prop(optional_no_strip)]
    invalid: Option<Signal<bool>>,
    /// Opens the visual thread editor.
    #[prop(optional_no_strip)]
    on_thread: Option<Callback<()>>,
    /// Sends the post.
    #[prop(optional)]
    on_submit: Option<Callback<()>>,
    /// Leaves the composer.
    #[prop(optional)]
    on_cancel: Option<Callback<()>>,
) -> impl IntoView {
    let count = move || crate::pages::compose::thread::count(&text.get());
    let over = move || !thread_mode.get() && count() > limit.get();
    let invalid = move || invalid.is_some_and(|invalid| invalid.get());
    let manual = move || extra.map_or(0, |extra| extra.get());
    let threading = move || thread_mode.get() && manual() > 0;
    let postable = move || {
        !busy.get() && !text.get().trim().is_empty() && on_submit.is_some() && !over() && !invalid()
    };

    view! {
        <div class="flex flex-col gap-2">
            <Show when=move || { over() && !thread_mode.get() }>
                <div class="alert alert-warning flex-wrap items-center gap-2 py-2 text-sm" role="alert">
                    <span>
                        "This server allows "
                        {move || limit.get()}
                        " characters. This draft is "
                        {move || count()}
                        "."
                    </span>
                    <button
                        class="btn btn-sm"
                        disabled=move || on_thread.is_none()
                        on:click=move |_| {
                            if let Some(on_thread) = on_thread {
                                on_thread.run(());
                            }
                        }
                    >
                        "Open thread editor"
                    </button>
                </div>
            </Show>
            <Show when=move || { thread_mode.get() && invalid() }>
                <p class="text-sm text-warning" role="alert">
                    "One page is still over the server limit. Split it before posting."
                </p>
            </Show>
            <Show when=move || { !thread_mode.get() && !over() && on_thread.is_some() }>
                <button
                    type="button"
                    class="btn btn-ghost btn-sm w-fit"
                    on:click=move |_| {
                        if let Some(on_thread) = on_thread {
                            on_thread.run(());
                        }
                    }
                >
                    "Add a thread"
                </button>
            </Show>
            <Show when=move || { threading() && !over() }>
                <p class="text-xs opacity-60">
                    {move || format!(
                        "This will go out as a thread of {} posts.",
                        manual() + 1,
                    )}
                </p>
            </Show>
            <div class="flex items-center gap-2">
            <button
                class="btn btn-primary btn-sm"
                disabled=move || !postable()
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
                } else if over() {
                    "Too long".to_owned()
                } else if threading() {
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
        </div>
    }
}
