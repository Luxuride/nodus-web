//! Mentions, boosts, favourites and follows, newest first.
//!
//! A notification is a pointer to something else, so every row renders the account that
//! caused it and — when the instance sent one — the status it is about. The row is the
//! link; nothing here fetches a status the instance already inlined.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::{Notification, NotificationKind};

use super::format::{display_name, relative_time};
use super::status::StatusCard;
use crate::state::bridge::actions;

/// What kind of attention this was.
fn describe(kind: NotificationKind) -> &'static str {
    match kind {
        NotificationKind::Follow => "followed you",
        NotificationKind::Reblog => "boosted your post",
        NotificationKind::Favourite => "favourited your post",
        NotificationKind::Mention => "mentioned you",
        NotificationKind::Poll => "A poll you voted in has ended",
        NotificationKind::Status => "posted",
        NotificationKind::AdminSignUp => "A new account signed up",
        NotificationKind::AdminReport => "A report needs review",
    }
}

/// The notifications list.
#[component]
pub fn Notifications() -> impl IntoView {
    let (items, set_items) = signal(Vec::<Notification>::new());
    let (cursor, set_cursor) = signal(None::<String>);
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(None::<String>);

    Effect::new(move |_| {
        spawn_local(async move {
            match actions::notifications(None).await {
                Ok(page) => {
                    set_items.set(page.items);
                    set_cursor.set(page.next_max_id);
                }
                Err(message) => set_error.set(Some(message)),
            }
        });
    });

    let more = move |_| {
        let after = cursor.get_untracked();
        set_loading.set(true);
        spawn_local(async move {
            match actions::notifications(after.as_deref()).await {
                Ok(page) => {
                    set_items.update(|items| items.extend(page.items));
                    set_cursor.set(page.next_max_id);
                }
                Err(message) => set_error.set(Some(message)),
            }
            set_loading.set(false);
        });
    };

    view! {
        <section class="flex flex-col gap-3">
            <Show when=move || cursor.get().is_some()>
                <button
                    class="btn btn-ghost btn-sm w-fit"
                    disabled=move || loading.get()
                    on:click=more
                >
                    "Load more"
                </button>
            </Show>

            <Show when=move || !loading.get() && items.get().is_empty()>
                <p class="text-sm opacity-60">"Nothing yet."</p>
            </Show>

            <ul class="flex flex-col gap-3">
                <For each=move || items.get() key=|notification| notification.id.clone() let:notification>
                    {
                        let who = notification.account.clone();
                        let heading = display_name(&who.display_name, &who.acct);
                        let when = relative_time(&notification.created_at);
                        let said = describe(notification.kind);
                        let status = notification.status.clone();
                        let link = format!("/accounts/{}", who.id);
                        view! {
                            <li class="card border border-base-300 bg-base-100 shadow-sm">
                                <div class="card-body gap-2 p-4">
                                    <div class="flex items-center gap-2">
                                        <a class="avatar" href=link>
                                            <div class="w-8 rounded-full bg-base-300">
                                                <img
                                                    src=who.avatar.clone()
                                                    alt=""
                                                    referrerpolicy="no-referrer"
                                                />
                                            </div>
                                        </a>
                                        <p class="min-w-0 flex-1 truncate text-sm">
                                            <a class="font-semibold hover:underline" href=format!("/accounts/{}", who.id)>
                                                {heading}
                                            </a>
                                            <span class="opacity-70">{format!(" {said}")}</span>
                                        </p>
                                        <time class="text-xs opacity-50" datetime=notification.created_at.clone()>
                                            {when}
                                        </time>
                                    </div>
                                    {status
                                        .map(|status| {
                                            view! {
                                                <ul class="flex flex-col gap-2">
                                                    <StatusCard status=status/>
                                                </ul>
                                            }
                                        })}
                                </div>
                            </li>
                        }
                    }
                </For>
            </ul>

            <Show when=move || error.get().is_some()>
                <p class="text-error text-xs">{move || error.get().unwrap_or_default()}</p>
            </Show>
        </section>
    }
}
