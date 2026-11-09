//! Mentions, boosts, favourites and follows, newest first.
//!
//! The screen itself is `nodus_view::pages::notifications`: a notification is a pointer to
//! something else, so every row draws the account that caused it and — when the instance sent
//! one — its own status. What stays here is what that crate may not have: the page store this
//! screen reads, the transport that answers a request, and the router that opens a thread.
//!
//! The pending-request strip is one such piece: it re-reads the queue on its own, which is a
//! read of the instance, so it arrives at the page as a slot instead of living inside it.

use std::sync::Arc;

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::Status;
use nodus_sdk::page::{PageItems, PageKind};

use crate::router;
use crate::routes::format::relative_time;
use crate::routes::status::{StatusCard, thread_route};
use crate::state::alive::Alive;
use crate::state::bridge::{actions, pages, revision};

mod pending;

use nodus_view::pages::notifications::{FollowRequestActions, NotificationsPage};
use pending::PendingRequests;

/// The notifications list.
#[component]
pub fn Notifications() -> impl IntoView {
    let revision = revision();
    let (loading, set_loading) = signal(!pages::held(&PageKind::Notifications));
    let (error, set_error) = signal(None::<String>);
    let alive = Alive::new();

    // The list is read from the SDK, not kept here: a second copy in this component is
    // exactly the drift the page store exists to remove.
    let held = move || {
        revision.get();
        pages::items(&PageKind::Notifications, |items| match items {
            PageItems::Notifications(list) => Some(list),
            _ => None,
        })
    };
    let items = move || held().map(|(list, _)| list).unwrap_or_default();
    let cursor = move || held().and_then(|(_, cursor)| cursor);

    Effect::new(move |_| {
        crate::screen::spawn_with(set_error, set_loading, pages::ensure(PageKind::Notifications), |_| ());
    });

    // A request that was just answered must disappear, so the list is re-read rather than
    // patched: the instance is the one that knows whether it was accepted.
    let reload = Callback::new(move |()| {
        spawn_local(async move {
            // A stale-marked page refetches: the decision was made on the instance.
            if let Err(message) = pages::ensure_after(PageKind::Notifications, None).await {
                alive.if_alive(|| set_error.set(Some(message)));
            }
        });
    });

    // The cursor the button was drawn with travels with the press, so a second page cannot be
    // read from a cursor the reader never had.
    let more = Callback::new(move |after: String| {
        set_loading.set(true);
        spawn_local(async move {
            if let Err(message) = pages::ensure_after(PageKind::Notifications, Some(after)).await {
                alive.if_alive(|| set_error.set(Some(message)));
            }
            alive.if_alive(|| set_loading.set(false));
        });
    });

    // One answer in flight at a time: the row asks the instance, the list is re-read, and the
    // buttons stay disabled until that round trip has finished.
    let (answering, set_answering) = signal(false);
    let answer = Callback::new(move |(account_id, accept): (String, bool)| {
        set_answering.set(true);
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
                    reload.run(());
                    set_answering.set(false);
                }),
                Err(message) => alive.if_alive(|| {
                    set_error.set(Some(message));
                    set_answering.set(false);
                }),
            }
        });
    });

    // The two pieces the page cannot draw: a status card with its verbs, and the controls that
    // answer a request. Both are this crate's own components, so a mounted row keeps the
    // router and the bus it already had.
    let status_card =
        Callback::new(move |status: Status| view! { <StatusCard status=status/> }.into_any());
    let request_actions = Callback::new(move |account_id: String| {
        view! {
            <FollowRequestActions
                account_id=account_id
                busy=answering.into()
                on_answer=answer
            />
        }
        .into_any()
    });

    let age_of = Callback::new(move |created_at: String| relative_time(&created_at));
    let on_open = Callback::new(move |id: String| router::navigate(thread_route(&id)));
    let banner: ChildrenFn =
        Arc::new(move || view! { <PendingRequests set_error=set_error/> }.into_any());

    view! {
        <NotificationsPage
            notifications=Signal::derive(items)
            cursor=Signal::derive(cursor)
            loading=loading.into()
            error=error.into()
            age_of=age_of
            on_more=more
            on_open=on_open
            status_card=status_card
            request_actions=request_actions
            banner=banner
        />
    }
}
