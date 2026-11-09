//! The follow requests waiting for an answer: reading them, and sending one.
//!
//! Mastodon delivers each request as a notification, but a request is a decision rather than
//! an event, so it is listed once above the queue and disappears when answered. The strip
//! itself is `nodus_view::pages::notifications::RequestBanner`; what stays here is the read
//! of the instance, the answer, and the re-read that follows it.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::Account;
use nodus_view::pages::notifications::RequestBanner;

use crate::router::{self, Route};
use crate::state::alive::Alive;
use crate::state::bridge::actions;

/// The requests, empty when there are none.
#[component]
pub fn PendingRequests(set_error: WriteSignal<Option<String>>) -> impl IntoView {
    let (requests, set_requests) = signal(Vec::<Account>::new());
    let (pending, set_pending) = signal(0_u32);
    let (busy, set_busy) = signal(false);
    let alive = Alive::new();

    // Reloading is a counter rather than a closure: the answer handler cannot call a
    // closure that is still being defined, and an effect on the counter re-reads the list.
    let refresh = move || set_pending.update(|count| *count += 1);

    Effect::new(move |_| {
        pending.get();

        spawn_local(async move {
            // Quiet on failure: this list is a supplement to the notifications the reader
            // came for, and a server that refuses it must not paint an error over them. An
            // empty list renders nothing, which is the honest outcome.
            if let Ok(found) = actions::follow_requests().await {
                alive.if_alive(|| set_requests.set(found));
            }
        });
    });

    let answer = Callback::new(move |(account_id, accept): (String, bool)| {
        set_busy.set(true);
        spawn_local(async move {
            let result = if accept {
                actions::authorize_follow_request(&account_id)
                    .await
                    .map(drop)
            } else {
                actions::reject_follow_request(&account_id).await
            };
            match result {
                Ok(()) => alive.if_alive(|| {
                    // The instance is the one that knows whether the request was accepted, so
                    // the queue is re-read rather than patched.
                    refresh();
                    set_busy.set(false);
                }),
                Err(message) => alive.if_alive(|| {
                    set_error.set(Some(message));
                    set_busy.set(false);
                }),
            }
        });
    });

    let open = Callback::new(move |id: String| router::navigate(Route::Account(id)));

    view! {
        <RequestBanner
            requests=Signal::derive(move || requests.get())
            busy=busy.into()
            on_answer=answer
            on_open=open
        />
    }
}
