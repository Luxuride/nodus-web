//! The follow requests waiting for an answer, above the notifications they arrived as.
//!
//! Mastodon delivers each request as a notification, but a request is a decision rather than
//! an event: it is listed once, above the queue, and disappears when it is answered. The strip
//! keeps its own box so it reads as a pending list and not as one more thing that happened.
//!
//! What it shows is the caller's list, so one piece of markup paints a request the client has
//! just read and one the server read a moment earlier. It renders nothing when the list is
//! empty, and it needs no instance to do that.
//!
//! Both links carry the account's address, so the server's paint is a working profile link and
//! the client cancels the navigation and routes in place. Only the name is a routing target:
//! the avatar keeps the plain link on both paints, as it always has.

use leptos::prelude::*;
use nodus_instance::model::Account;

use super::request::{FollowRequestActions, FollowRequestActionsProps};
use crate::display_name;

/// The pending-request strip, or nothing when nothing is pending.
#[component]
pub fn RequestBanner(
    /// The requests waiting, as the caller last read them.
    requests: Signal<Vec<Account>>,
    /// Whether an answer is already on its way.
    #[prop(optional)]
    busy: Option<Signal<bool>>,
    /// Sends `(account id, accepted)`. Absent on the server.
    #[prop(optional)]
    on_answer: Option<Callback<(String, bool)>>,
    /// Follows an account link inside the app.
    #[prop(optional)]
    on_open: Option<Callback<String>>,
) -> impl IntoView {
    view! {
        <Show when=move || !requests.get().is_empty()>
            <section class="card border border-base-300 bg-base-100 shadow-sm">
                <div class="card-body gap-2 p-4">
                    <h2 class="card-title text-base">"Follow requests"</h2>
                    <ul class="flex flex-col gap-2">
                        <For
                            each=move || requests.get()
                            key=|account| account.id.clone()
                            let:account
                        >
                            {
                                let account_id = account.id.clone();
                                let name = display_name(&account.display_name, &account.acct);
                                let href = format!("/accounts/{account_id}");
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
                                            on:click=move |event| {
                                                open_account(&event, account_id.clone(), on_open);
                                            }
                                        >
                                            {name}
                                        </a>
                                        {FollowRequestActions(FollowRequestActionsProps {
                                            account_id: account.id.clone(),
                                            busy,
                                            on_answer,
                                        })}
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

/// Follows an account link inside the app when there is a callback for it.
///
/// Without one the press is left alone: the `href` is the whole answer, which is what the
/// server's paint relies on (it emits no listener at all). The non-wasm build keeps a use of
/// `event` so the listener argument is not reported as unused on the target that has no
/// `prevent_default`.
fn open_account(event: &leptos::ev::MouseEvent, id: String, on_open: Option<Callback<String>>) {
    #[cfg(not(target_arch = "wasm32"))]
    let _ = event;
    if let Some(on_open) = on_open {
        #[cfg(target_arch = "wasm32")]
        event.prevent_default();
        on_open.run(id);
    }
}
