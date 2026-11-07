//! One status, with the actions a reader can take on it.
//!
//! The card owns the copy it renders: an action returns the status as the instance
//! now reports it, and the card replaces its local copy with that answer. State the SDK
//! owns (which timeline is loaded, where the cursor is) is untouched, because a
//! favourite is not a change to the list — it is a change to one status.

use leptos::prelude::*;

mod thread;

pub use self::thread::Thread;
use nodus_instance::model::Status;

use super::format::{display_name, relative_time};
use crate::router::{self, Route};

mod body;
mod footer;
mod poll;
mod reply;

use body::StatusBody;
use footer::CardFooter;

/// A status card: who wrote it, what it says, and what can be done with it.
#[component]
pub fn StatusCard(status: Status) -> impl IntoView {
    // A boost is shown as the boosted status with a line saying who boosted it: the
    // wrapper carries the booster's account and an empty body, so rendering the wrapper
    // would show a nameless post with no text.
    let booster = status
        .reblog
        .as_ref()
        .map(|_| format!("{} boosted", status.account.acct));
    // A signal, not a local: `Show`'s children closure must be `Fn`, and a `String`
    // captured by value would make it `FnOnce`.
    let (booster_text, _) = signal(booster.unwrap_or_default());
    let initial = match status.reblog {
        Some(inner) => *inner,
        None => status,
    };
    let (shown, set_shown) = signal(initial);
    let (busy, set_busy) = signal(false);
    let (error, set_error) = signal(None::<String>);

    // One copy for the link targets: a signal read inside `href` would rebuild the
    // anchor on every action, and the browser would lose focus on the link.
    let author_href = move || Route::Account(shown.get_untracked().account.id.clone()).path();
    let thread_id = move |status: &Status| Route::Thread(status.id.clone());

    view! {
        <li class="card border border-base-300 bg-base-100 shadow-sm">
            <div class="card-body gap-3 p-4">
                <Show when=move || !booster_text.get().is_empty()>
                    <p class="text-xs opacity-60">{move || booster_text.get()}</p>
                </Show>
                <header class="flex items-center gap-3">
                    <a
                        class="avatar"
                        href=author_href
                        on:click=move |event| {
                            router::link(
                                event,
                                Route::Account(shown.get_untracked().account.id.clone()),
                            );
                        }
                    >
                        <div class="w-10 rounded-full bg-base-300">
                            <img
                                src=move || shown.get().account.avatar.clone()
                                alt=""
                                loading="lazy"
                                referrerpolicy="no-referrer"
                            />
                        </div>
                    </a>
                    <div class="min-w-0 flex-1">
                        <a
                            class="block truncate font-semibold leading-tight hover:underline"
                            href=author_href
                            on:click=move |event| {
                                router::link(
                                    event,
                                    Route::Account(shown.get_untracked().account.id.clone()),
                                );
                            }
                        >
                            {move || {
                                let account = shown.get().account;
                                display_name(&account.display_name, &account.acct)
                            }}
                        </a>
                        <a
                            class="block truncate text-sm opacity-60 hover:underline"
                            href=author_href
                            on:click=move |event| {
                                router::link(
                                    event,
                                    Route::Account(shown.get_untracked().account.id.clone()),
                                );
                            }
                        >
                            {move || shown.get().account.acct}
                        </a>
                    </div>
                    <a
                        class="text-xs whitespace-nowrap opacity-50 hover:underline"
                        href=move || thread_id(&shown.get_untracked()).path()
                        on:click=move |event| {
                            router::link(event, thread_id(&shown.get_untracked()));
                        }
                    >
                        {move || relative_time(&shown.get().created_at)}
                    </a>
                </header>

                <StatusBody shown=shown set_shown=set_shown/>

                <CardFooter shown=shown set_shown=set_shown busy=busy set_busy=set_busy set_error=set_error/>
                <Show when=move || error.get().is_some()>
                    <p class="text-error text-xs">{move || error.get().unwrap_or_default()}</p>
                </Show>
            </div>
        </li>
    }
}
