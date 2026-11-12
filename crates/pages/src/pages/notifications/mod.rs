//! The notifications screen: mentions, boosts, favourites and follows, newest first.
//!
//! A notification is a pointer to something else, so every row draws the account that caused
//! it and — when the instance sent one — the status it is about. Nothing here fetches a status
//! the instance already inlined, and nothing here reads the clock: the age of a row arrives
//! through `age_of`, so one page paints from the server and mounts on the client.
//!
//! The screen is a page and three pieces it alone uses: the row ([`NotificationRow`]), the
//! accept/refuse controls ([`FollowRequestActions`]) and the pending-request strip
//! ([`RequestBanner`]). The strip is a slot rather than part of this view because filling it
//! reads the instance, which a page may not do; it sits above the list and outside the list's
//! container, so a queue with nothing in it still shows the requests waiting on it.

mod banner;
mod request;
mod row;

pub use banner::RequestBanner;
pub use request::FollowRequestActions;
pub use row::NotificationRow;

use row::NotificationRowProps;

use leptos::prelude::*;
use nodus_client_core::model::{Notification, Status};

use crate::{PageHeader, Screen};

/// The notifications list, with its paging control, empty state and error line.
#[component]
pub fn NotificationsPage(
    /// The rows, newest first, as the caller last read them.
    notifications: Signal<Vec<Notification>>,
    /// The cursor of the page after this one, when the instance has more.
    cursor: Signal<Option<String>>,
    /// Whether a read is already on its way.
    loading: Signal<bool>,
    /// The last read's refusal, when it had one.
    error: Signal<Option<String>>,
    /// Formats a timestamp as an age. Absent when the caller has no clock to lend.
    #[prop(optional)]
    age_of: Option<Callback<String, String>>,
    /// Asks for the page after the given cursor. Absent on the server, where the control
    /// keeps its box and is disabled.
    #[prop(optional)]
    on_more: Option<Callback<String>>,
    /// Opens the thread of the status a notification points at.
    #[prop(optional)]
    on_open: Option<Callback<String>>,
    /// The client's own interactive status card, per inlined status.
    #[prop(optional)]
    status_card: Option<Callback<Status, AnyView>>,
    /// The client's own accept/refuse controls, per follow request.
    #[prop(optional)]
    request_actions: Option<Callback<String, AnyView>>,
    /// The pending-request strip, drawn above the list.
    #[prop(optional)]
    banner: Option<ChildrenFn>,
) -> impl IntoView {
    // The rows' ages are the caller's clock, asked once per row so the page reads no time of
    // its own.
    let age_of_created = move |created_at: &str| {
        age_of.map_or_else(String::new, |age_of| age_of.run(created_at.to_owned()))
    };

    // The control carries the cursor it was drawn with, so a press cannot page from a cursor
    // the reader never saw. Nothing is sent when there is no verb or no next page — the button
    // is disabled in the first case, absent in the second.
    let more = move |_| {
        let Some(on_more) = on_more else {
            return;
        };
        let Some(after) = cursor.get_untracked() else {
            return;
        };
        on_more.run(after);
    };

    view! {
        <Screen>
            <PageHeader
                title="Notifications"
                description="Mentions, boosts, favourites and follow requests for this account."
            />
            {banner.map(|banner| banner())}

            // The paging control sits above the list on this screen, where the client has
            // always drawn it: a press appends to the list, so the control stays where the
            // reader last used it instead of running away down the column.
            <Show when=move || cursor.get().is_some()>
                <button
                    class="btn btn-ghost btn-sm w-fit"
                    disabled=move || loading.get() || on_more.is_none()
                    on:click=more
                >
                    "Load more"
                </button>
            </Show>

            // An empty queue is a state, not an error: it is said once, after the first read
            // has finished, and never while the list is still on its way.
            <Show when=move || !loading.get() && notifications.get().is_empty()>
                <p class="text-sm opacity-60">"Nothing yet."</p>
            </Show>

            <ul class="flex flex-col gap-3">
                <For
                    each=move || notifications.get()
                    key=|notification| notification.id.clone()
                    let:notification
                >
                    {
                        let age = age_of_created(&notification.created_at);
                        NotificationRow(NotificationRowProps {
                            notification,
                            age,
                            age_of,
                            on_open,
                            status_card,
                            request_actions,
                        })
                    }
                </For>
            </ul>

            <Show when=move || error.get().is_some()>
                <p class="text-error text-xs">{move || error.get().unwrap_or_default()}</p>
            </Show>
        </Screen>
    }
}
