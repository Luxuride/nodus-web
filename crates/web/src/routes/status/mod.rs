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
    // The shared line is frozen at mount: the author, the addresses and the age do not
    // change when a verb answers, so a signal would rebuild the anchors for nothing.
    let header = initial.clone();
    let (shown, set_shown) = signal(initial);
    let (busy, set_busy) = signal(false);
    let (error, set_error) = signal(None::<String>);

    // One copy for the link targets: a signal read inside `href` would rebuild the
    // anchor on every action, and the browser would lose focus on the link.
    let thread_id = move |status: &Status| thread_route(&status.id);
    let age = super::format::relative_time(&header.created_at);
    let author_path = Route::Account(header.account.id.clone()).path();
    let thread_path = thread_route(&header.id).path();
    let navigate = Callback::new(move |path: String| router::navigate(Route::from_path(&path)));

    view! {
        <li class="card border border-base-300 bg-base-100 shadow-sm">
            <div class="card-body gap-3 p-4">
                <Show when=move || !booster_text.get().is_empty()>
                    <p class="text-xs opacity-60">{move || booster_text.get()}</p>
                </Show>
                <nodus_view::status_header::StatusHeader
                    status=header
                    age=age
                    author_path=author_path
                    thread_path=thread_path
                    on_navigate=navigate
                />

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
