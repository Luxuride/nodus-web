//! Posts queued for later.
//!
//! This screen is the queue, not a way to add to it: a post is scheduled from the box
//! that also holds its text, so there is one place to write and one place to look at what
//! is waiting. Everything here can do is read the queue and cancel an entry.
//!
//! The screen is `nodus_view::pages::scheduled`; what stays here is the store read, the
//! re-read after a cancel, and the client's clock.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_sdk::page::{PageItems, PageKind};
use nodus_view::pages::scheduled::ScheduledPage;

use crate::routes::format::relative_time;
use crate::state::alive::Alive;
use crate::state::bridge::{actions, dispatch, pages, revision};

/// The queue of posts waiting to be published.
#[component]
pub fn Scheduled() -> impl IntoView {
    let revision = revision();
    let (loading, set_loading) = signal(!pages::held(&PageKind::Scheduled));
    let (error, set_error) = signal(None::<String>);
    let alive = Alive::new();

    let items = move || {
        revision.get();
        pages::items(&PageKind::Scheduled, |items| match items {
            PageItems::Scheduled(list) => Some(list),
            _ => None,
        })
        .map(|(list, _)| list)
        .unwrap_or_default()
    };

    let reload = move || {
        let alive = alive;
        spawn_local(async move {
            // A queue that changed on another device must be re-read, so the page is
            // marked stale before asking rather than trusted.
            dispatch(nodus_sdk::Command::PageStale {
                kind: PageKind::Scheduled,
            });
            match pages::ensure(PageKind::Scheduled).await {
                Ok(_) => alive.if_alive(|| set_loading.set(false)),
                Err(message) => alive.if_alive(|| {
                    set_error.set(Some(message));
                    set_loading.set(false);
                }),
            }
        });
    };

    Effect::new(move |_| reload());

    let cancel = Callback::new(move |id: String| {
        spawn_local(async move {
            match actions::cancel_scheduled(&id).await {
                Ok(()) => reload(),
                Err(message) => alive.if_alive(|| set_error.set(Some(message))),
            }
        });
    });

    // The row shows the instance's own stamp next to the age, so the caller hands both: the
    // page keeps the stamp when it has no clock of its own to lend.
    let age_of = Callback::new(move |scheduled_at: String| {
        format!("{scheduled_at} · {}", relative_time(&scheduled_at))
    });

    view! {
        <ScheduledPage
            items=Signal::derive(items)
            loading=loading.into()
            error=error.into()
            age_of=age_of
            on_cancel=cancel
        />
    }
}
