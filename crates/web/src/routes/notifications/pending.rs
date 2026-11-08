//! Follow requests waiting for an answer.
//!
//! Mastodon delivers each request as a notification, but a request is a decision rather
//! than a notification, so it is listed once and disappears when answered.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::Account;

use super::request::FollowRequestActions;
use crate::router::{self, Route};
use crate::routes::format::display_name;
use crate::state::alive::Alive;
use crate::state::bridge::actions;

/// The requests, empty when there are none.
#[component]
pub fn PendingRequests(set_error: WriteSignal<Option<String>>) -> impl IntoView {
    let (requests, set_requests) = signal(Vec::<Account>::new());
    let (pending, set_pending) = signal(0_u32);

    // Reloading is a counter rather than a closure: the answer handler cannot call a
    // closure that is still being defined, and an effect on the counter re-reads the list.
    let refresh = move || set_pending.update(|count| *count += 1);
    let alive = Alive::new();

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

    view! {
        <Show when=move || !requests.get().is_empty()>
            <section class="card border border-base-300 bg-base-100 shadow-sm">
                <div class="card-body gap-2 p-4">
                    <h2 class="card-title text-base">"Follow requests"</h2>
                    <ul class="flex flex-col gap-2">
                        <For each=move || requests.get() key=|account| account.id.clone() let:account>
                            {
                                let route = Route::Account(account.id.clone());
                                let href = route.path();
                                view! {
                                    <li class="flex items-center gap-2">
                                        <a class="avatar" href=href.clone()>
                                            <div class="w-8 rounded-full bg-base-300">
                                                <img
                                                    src=account.avatar.clone()
                                                    alt=""
                                                    referrerpolicy="no-referrer"
                                                />
                                            </div>
                                        </a>
                                        <a
                                            class="min-w-0 flex-1 truncate text-sm hover:underline"
                                            href=href
                                            on:click=move |event| router::link(event, route.clone())
                                        >
                                            {display_name(&account.display_name, &account.acct)}
                                        </a>
                                        <FollowRequestActions
                                            account_id=account.id.clone()
                                            on_done=Callback::new(move |()| refresh())
                                            set_error=set_error
                                        />
                                    </li>
                                }
                            }
                        </For>
                    </ul>
                </div>
            </section>
        </Show>
    }
}
