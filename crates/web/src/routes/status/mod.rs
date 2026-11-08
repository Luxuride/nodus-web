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
pub use public::PublicThread;

use super::format::{display_name, relative_time};
use crate::router::{self, Route};

mod body;
mod chips;
mod footer;
mod menu;
mod poll;
mod public;
mod reply;
pub(crate) mod sensitive;
mod share;

use body::StatusBody;
use footer::CardFooter;

/// The route to one status, host included.
///
/// A thread link is also a shareable link, so the host travels with the id: a link the app
/// builds itself must not be the one link it cannot read back.
pub(super) fn thread_route(id: &str) -> Route {
    Route::Thread {
        host: crate::state::bridge::session::remembered_host().unwrap_or_default(),
        id: id.to_owned(),
    }
}

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
    let thread_id = move |status: &Status| thread_route(&status.id);

    view! {
        <li class="card border border-base-300 bg-base-100 shadow-sm">
            <div class="card-body gap-3 p-4">
                <Show when=move || !booster_text.get().is_empty()>
                    <p class="text-xs opacity-60">{move || booster_text.get()}</p>
                </Show>
                {status_header(shown)}

                // The whole body opens the post: reading it in place is the point of a
                // timeline, and reaching for the timestamp to do it is a second step nobody
                // wants. Links inside the body keep their own destination — the delegated
                // listener already knows a prevented click when it sees one.
                <div
                    class="cursor-pointer"
                    on:click=move |event: web_sys::MouseEvent| {
                        if event.default_prevented() || inside_control(&event) {
                            return;
                        }
                        router::link(event, thread_id(&shown.get_untracked()));
                    }
                >
                    <StatusBody shown=shown set_shown=set_shown/>
                </div>

                <CardFooter shown=shown set_shown=set_shown busy=busy set_busy=set_busy set_error=set_error/>
                <Show when=move || error.get().is_some()>
                    <p class="text-error text-xs">{move || error.get().unwrap_or_default()}</p>
                </Show>
            </div>
        </li>
    }
}

/// The author line, kept in its own function so `StatusCard` stays under the lint's
/// line budget: the card grew an image grid, a poll and a click-to-open wrapper, and a
/// hundred lines of markup in one body is where a component stops being readable.
fn status_header(shown: ReadSignal<Status>) -> impl IntoView {
    // Computed once: a signal read inside `href` would rebuild the anchor on every
    // action, and the browser would lose the link's focus.
    let author_href = move || Route::Account(shown.get_untracked().account.id.clone()).path();
    let thread_id = move |status: &Status| thread_route(&status.id);
    view! {
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
                                width="40"
                                height="40"
                                decoding="async"
                                class="h-10 w-10 rounded-full object-cover"
                                src=move || shown.get().account.avatar.clone()
                                alt=""
                                loading="lazy"
                                referrerpolicy="no-referrer"
                            />
                        </div>
                    </a>
                    <div class="flex min-w-0 flex-1 flex-wrap items-baseline gap-x-1">
                        <a
                            class="max-w-full truncate font-semibold leading-tight hover:underline"
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
                            class="max-w-full truncate text-sm opacity-60 hover:underline"
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
    }
}

/// Whether the press landed on something that owns its own click.
///
/// The body wrapper opens the post on any press, which is what a reader expects from the
/// text — but a picture, a poll option, a link or the "show content" summary all have a
/// destination of their own, and stealing it made the card feel broken.
fn inside_control(event: &web_sys::MouseEvent) -> bool {
    use wasm_bindgen::JsCast as _;

    event
        .target()
        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
        .and_then(|element| {
            element
                .closest("a, button, input, textarea, select, summary, video, audio")
                .ok()
                .flatten()
        })
        .is_some()
}
