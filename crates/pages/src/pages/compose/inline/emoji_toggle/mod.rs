//! The emoticon button that opens the picker.
//!
//! The button is markup; the document listener that closes the panel on a press elsewhere
//! is DOM, so it stays in the client, which wraps this button in the host the listener
//! checks and mounts the picker beside it.

use leptos::prelude::*;

/// The emoticon, which sits with the counter rather than with the compose-your-post row:
/// it edits the body, and the left group is about what kind of post this is.
#[component]
pub fn EmojiToggleButton(
    /// Whether the picker is showing.
    #[prop(into)]
    open: Signal<bool>,
    /// Disables the button while a post is in flight.
    #[prop(into)]
    busy: Signal<bool>,
    /// Toggles the picker.
    #[prop(optional)]
    on_toggle: Option<Callback<()>>,
) -> impl IntoView {
    view! {
        <button
            type="button"
            data-emoji-toggle
            class="btn btn-outline btn-sm px-2 text-lg leading-none"
            class:bg-base-200=move || open.get()
            title="Insert an emoji"
            aria-expanded=move || open.get().to_string()
            disabled=move || busy.get() || on_toggle.is_none()
            on:click=move |_| {
                if let Some(on_toggle) = on_toggle {
                    on_toggle.run(());
                }
            }
        >
            <span class="text-xl leading-none">"☺"</span>
        </button>
    }
}
