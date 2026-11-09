//! What the author of a status can do to it: edit it, or delete it.
//!
//! The two controls are drawn here because the server paints the card too, and a control that
//! moved between the paints would move the action row under the reader. Who may press them is
//! not this crate's question: the caller only mounts the menu for the author's own statuses,
//! and passes the verbs; without them the controls keep their boxes and lose only their press.

use leptos::prelude::*;

/// Edit and delete, for the author.
///
/// `id` is the status id, which is also the edit address. With no `on_edit` the anchor is the
/// edit page itself; with no `on_delete` the button is disabled rather than absent, so the row
/// keeps its shape on both paints.
#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "a Leptos component receives its props by value"
)]
pub fn StatusMenu(
    /// The status this menu belongs to.
    id: String,
    /// Opens the editor inside the app. Absent where the anchor is the whole answer.
    #[prop(optional)]
    on_edit: Option<Callback<()>>,
    /// Deletes the status after the caller has confirmed. Absent where nothing can be deleted.
    #[prop(optional)]
    on_delete: Option<Callback<()>>,
) -> impl IntoView {
    let disabled = on_delete.is_none();

    view! {
        <a class="btn btn-ghost btn-xs" href=format!("/compose/{id}") on:click=move |event| follow_edit(&event, on_edit)>
            "Edit"
        </a>
        <button
            class="btn btn-ghost btn-xs text-error"
            disabled=disabled
            on:click=move |_| {
                if let Some(on_delete) = on_delete {
                    on_delete.run(());
                }
            }
        >
            "Delete"
        </button>
    }
}

/// Follows the edit anchor when there is a callback for it.
///
/// Without one the press is left alone: the `href` is the whole answer, which is what the
/// server's paint relies on. The non-wasm build keeps a use of `event` so the listener
/// argument is not reported as unused on the target that has no `prevent_default`.
fn follow_edit(event: &leptos::ev::MouseEvent, on_edit: Option<Callback<()>>) {
    #[cfg(not(target_arch = "wasm32"))]
    let _ = event;
    if let Some(on_edit) = on_edit {
        #[cfg(target_arch = "wasm32")]
        event.prevent_default();
        on_edit.run(());
    }
}
