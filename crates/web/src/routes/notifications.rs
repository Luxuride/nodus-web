//! Mentions, boosts, favourites and follows, newest first.
//!
//! A notification is a pointer to something else, so every row renders the account that
//! caused it and — when the instance sent one — the status it is about. The row is the
//! link; nothing here fetches a status the instance already inlined.

use leptos::prelude::*;

use crate::ui::page::PageHeader;
use leptos::task::spawn_local;
use nodus_instance::model::Notification;

use crate::state::alive::Alive;
use crate::state::bridge::actions;

mod pending;
mod request;
mod row;

use pending::PendingRequests;
use row::NotificationRow;

/// The notifications list.
#[component]
pub fn Notifications() -> impl IntoView {
    let (items, set_items) = signal(Vec::<Notification>::new());
    let (cursor, set_cursor) = signal(None::<String>);
    // Starts true: until the first request is issued, "nothing yet" is a
    // guess, and a guess renders as an empty state that flickers.
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);
    let alive = Alive::new();

    Effect::new(move |_| {
        spawn_local(async move {
            match actions::notifications(None).await {
                Ok(page) => alive.if_alive(|| {
                    set_items.set(page.items);
                    set_cursor.set(page.next_max_id);
                }),
                Err(message) => alive.if_alive(|| set_error.set(Some(message))),
            }
        });
    });

    // A request that was just answered must disappear, so the list is re-read rather than
    // patched: the instance is the one that knows whether it was accepted.
    let reload = {
        Callback::new(move |()| {
            spawn_local(async move {
                if let Ok(page) = actions::notifications(None).await {
                    alive.if_alive(|| {
                        set_items.set(page.items);
                        set_cursor.set(page.next_max_id);
                    });
                }
            });
        })
    };

    let more = move |_| {
        let after = cursor.get_untracked();

        set_loading.set(true);
        spawn_local(async move {
            match actions::notifications(after.as_deref()).await {
                Ok(page) => alive.if_alive(|| {
                    set_items.update(|items| items.extend(page.items));
                    set_cursor.set(page.next_max_id);
                }),
                Err(message) => alive.if_alive(|| set_error.set(Some(message))),
            }
            alive.if_alive(|| set_loading.set(false));
        });
    };

    view! {
        <section class="flex flex-col gap-3">
            <PageHeader title="Notifications" description="Mentions, boosts, favourites and follow requests for this account."/>
            <PendingRequests set_error=set_error/>

            <Show when=move || cursor.get().is_some()>
                <button class="btn btn-ghost btn-sm w-fit" disabled=move || loading.get() on:click=more>
                    "Load more"
                </button>
            </Show>

            <Show when=move || !loading.get() && items.get().is_empty()>
                <p class="text-sm opacity-60">"Nothing yet."</p>
            </Show>

            <ul class="flex flex-col gap-3">
                <For each=move || items.get() key=|notification| notification.id.clone() let:notification>
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
