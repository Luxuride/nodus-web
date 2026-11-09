//! The one action every reader has on a post: a link to it.
//!
//! The button is shared so the server and the client put it in the same place in the action
//! row; what it does is the clipboard, which is the client's. A failed copy changes the label
//! back on the next paint, which is why the label arrives as a signal rather than as a bool.

use leptos::prelude::*;

/// Copies the post's shareable link.
///
/// With no `on_copy` the button keeps its box and is disabled: the server has no clipboard,
/// and a control that vanished there would shorten the row.
#[component]
pub fn ShareButton(
    /// Whether the last copy succeeded, and the label should read "Copied".
    #[prop(into)]
    copied: Signal<bool>,
    /// Writes the link. Absent where there is no clipboard.
    #[prop(optional)]
    on_copy: Option<Callback<()>>,
) -> impl IntoView {
    let disabled = on_copy.is_none();

    view! {
        <button
            class="btn btn-ghost btn-sm"
            disabled=disabled
            on:click=move |_| {
                if let Some(on_copy) = on_copy {
                    on_copy.run(());
                }
            }
        >
            {move || if copied.get() { "Copied" } else { "Link" }}
        </button>
    }
}
