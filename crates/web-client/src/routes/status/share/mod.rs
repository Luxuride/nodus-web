//! The one action every reader has on a post: a link to it.
//!
//! Split from the footer so the footer stays about the reader's own state (`favourite`,
//! `bookmark`) and this stays about the post's address.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_client_core::model::Status;

use crate::routes::feed::share;
use crate::state::alive::Alive;
use crate::state::bridge::with_sdk;

/// Copies the post's shareable link.
///
/// The label changes to "Copied" because a clipboard write is otherwise invisible: the
/// only feedback a silent success gives is that nothing happened.
#[component]
pub(super) fn ShareButton(
    shown: ReadSignal<Status>,
    set_error: WriteSignal<Option<String>>,
) -> impl IntoView {
    let alive = Alive::new();
    let (copied, set_copied) = signal(false);

    let copy = move || {
        let id = shown.get_untracked().id;
        // The host is what makes the link work for a reader who is not signed in: the post
        // is fetched from *that* instance, by them.
        let Some(host) = with_sdk(|sdk| sdk.state().session.get().map(|session| session.host))
        else {
            set_error.set(Some("Sign in to copy a link.".to_owned()));
            return;
        };
        spawn_local(async move {
            match share::copy_post_link(&host, &id).await {
                Ok(()) => {
                    set_error.set(None);
                    alive.if_alive(|| set_copied.set(true));
                }
                Err(message) => alive.if_alive(|| set_error.set(Some(message))),
            }
        });
    };

    view! {
        <nodus_ui::status_share::ShareButton
            copied=Signal::derive(move || copied.get())
            on_copy=Callback::new(move |()| copy())
        />
    }
}
