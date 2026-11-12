//! What the author of a status can do to it.
//!
//! Only rendered for the signed-in account's own statuses: the instance answers 403 for
//! anyone else, and showing a button that always fails is worse than showing none.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_client_core::model::Status;

use crate::router::{self, Route};
use crate::state::alive::Alive;
use crate::state::bridge::actions;

/// Edit and delete, for the author.
#[component]
pub fn StatusMenu(
    shown: ReadSignal<Status>,
    set_error: WriteSignal<Option<String>>,
) -> impl IntoView {
    let alive = Alive::new();
    let mine = move || {
        let status = shown.get();
        actions::viewer_id().is_some_and(|id| id == status.account.id)
    };

    let remove = move || {
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
                Err(message) => alive.if_alive(|| set_error.set(Some(message))),
            }
        });
    };

    view! {
        <Show when=mine>
            {move || {
                let id = shown.get().id;
                let for_edit = id.clone();
                view! {
                    <nodus_ui::status_menu::StatusMenu
                        id=id
                        on_edit=Callback::new(move |()| {
                            router::navigate(Route::ComposeEdit(for_edit.clone()));
                        })
                        on_delete=Callback::new(move |()| remove())
                    />
                }
            }}
        </Show>
    }
}
