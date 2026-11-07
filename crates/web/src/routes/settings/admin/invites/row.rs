//! One invite, with the only action it has.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::Invite;

use crate::state::bridge::{admin, with_sdk};

/// A link, what is left of it, and the button that ends it.
#[component]
pub(super) fn InviteRow(
    invite: Invite,
    set_items: WriteSignal<Vec<Invite>>,
    set_error: WriteSignal<Option<String>>,
) -> impl IntoView {
    // Read everything before moving the id out: a partial move ends the borrow.
    let link = invite_link(&invite.code);
    let left = invite.uses_left();
    let usable = invite.usable();
    let id = invite.id;
    let revoke = move |_| {
        let id = id.clone();
        spawn_local(async move {
            match admin::revoke_invite(&id).await {
                Ok(()) => set_items.update(|list| list.retain(|item| item.id != id)),
                Err(message) => set_error.set(Some(message)),
            }
        });
    };

    view! {
        <div class="flex flex-wrap items-center gap-2 rounded-box bg-base-200 p-2 text-sm">
            <code class="truncate text-xs">{link}</code>
            <span class="badge badge-ghost badge-sm">{left}</span>
            <Show
                when=move || usable
                fallback=|| view! { <span class="badge badge-error badge-sm">"revoked"</span> }
            >
                <span class="badge badge-success badge-sm">"open"</span>
            </Show>
            <button class="btn btn-ghost btn-xs ml-auto" on:click=revoke>
                "Revoke"
            </button>
        </div>
    }
}

/// The link a person can actually open: the code is appended to this instance's host.
fn invite_link(code: &str) -> String {
    with_sdk(|sdk| {
        sdk.state().session.get().map_or_else(
            String::new,
            |session| format!("https://{}/invite/{code}", session.host),
        )
    })
}
