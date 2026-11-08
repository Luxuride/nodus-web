//! Mentions, boosts, favourites and follows, newest first.
//!
//! A notification is a pointer to something else, so every row renders the account that
//! caused it and — when the instance sent one — the status it is about. The row is the
//! link; nothing here fetches a status the instance already inlined.

use leptos::prelude::*;

use crate::ui::page::PageHeader;
use leptos::task::spawn_local;

use crate::state::alive::Alive;
use crate::state::bridge::{pages, revision};
use nodus_sdk::page::{PageItems, PageKind};

mod pending;
mod request;
mod row;

use pending::PendingRequests;
use row::NotificationRow;

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
        spawn_local(async move {
            match pages::ensure(PageKind::Notifications).await {
                Ok(_) => alive.if_alive(|| set_loading.set(false)),
                Err(message) => alive.if_alive(|| {
                    set_error.set(Some(message));
                    set_loading.set(false);
                }),
            }
        });
    });

    // A request that was just answered must disappear, so the list is re-read rather than
    // patched: the instance is the one that knows whether it was accepted.
    let reload = {
        Callback::new(move |()| {
            spawn_local(async move {
                // A stale-marked page refetches: the decision was made on the instance.
                if let Err(message) = pages::ensure_after(PageKind::Notifications, None).await {
                    alive.if_alive(|| set_error.set(Some(message)));
                }
            });
        })
    };

    let more = move |_| {
        let after = cursor();
        set_loading.set(true);
        spawn_local(async move {
            if let Err(message) = pages::ensure_after(PageKind::Notifications, after).await {
                alive.if_alive(|| set_error.set(Some(message)));
            }
            alive.if_alive(|| set_loading.set(false));
        });
    };

    view! {
        <section class="flex flex-col gap-3">
            <PageHeader title="Notifications" description="Mentions, boosts, favourites and follow requests for this account."/>
            <PendingRequests set_error=set_error/>

            <Show when=move || cursor().is_some()>
                <button class="btn btn-ghost btn-sm w-fit" disabled=move || loading.get() on:click=more>
                    "Load more"
                </button>
            </Show>

            <Show when=move || !loading.get() && items().is_empty()>
                <p class="text-sm opacity-60">"Nothing yet."</p>
            </Show>

            <ul class="flex flex-col gap-3">
                <For each=items key=|notification| notification.id.clone() let:notification>
                    <NotificationRow
                        notification=notification
                        reload=reload
                        set_error=set_error
                    />
                </For>
            </ul>

            <Show when=move || error.get().is_some()>
                <p class="text-error text-xs">{move || error.get().unwrap_or_default()}</p>
            </Show>
        </section>
    }
}
