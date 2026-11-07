//! What the author of a status can do to it.
//!
//! Only rendered for the signed-in account's own statuses: the instance answers 403 for
//! anyone else, and showing a button that always fails is worse than showing none.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::Status;

use crate::router::{self, Route};
use crate::state::bridge::actions;

/// Edit and delete, for the author.
#[component]
pub fn StatusMenu(
    shown: ReadSignal<Status>,
    set_error: WriteSignal<Option<String>>,
) -> impl IntoView {
    let mine = move || {
        let status = shown.get();
        actions::viewer_id().is_some_and(|id| id == status.account.id)
    };

    let remove = move |_| {
        let status = shown.get_untracked();
        if !web_sys::window()
            .and_then(|window| window.confirm_with_message("Delete this status?").ok())
            .unwrap_or(false)
        {
            return;
        }
        spawn_local(async move {
            match actions::delete_status(&status.id).await {
                // The status has no page left to show, and every list holding it is stale.
                Ok(()) => router::navigate(Route::Home),
                Err(message) => set_error.set(Some(message)),
            }
        });
    };

    view! {
        <Show when=mine>
            {move || {
                let id = shown.get().id;
                let for_link = id.clone();
                let href = format!("/compose/{id}");
                view! {
                    <a
                        class="btn btn-ghost btn-xs"
                        href=href
                        on:click=move |event| {
                            router::link(event, Route::ComposeEdit(for_link.clone()));
                        }
                    >
                        "Edit"
                    </a>
                }
            }}
            <button class="btn btn-ghost btn-xs text-error" on:click=remove>
                "Delete"
            </button>
        </Show>
    }
}
