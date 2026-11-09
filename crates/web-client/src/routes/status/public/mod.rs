//! A post opened from a shared link, with no session.
//!
//! The screen is shared; this file owns what a visitor's link decides — which host to ask,
//! and what the two reads put in the page. Read-only on purpose: every verb in the card
//! needs a token, and pretending otherwise would offer buttons that cannot work.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::Status;
use nodus_view::pages::status::public::{HostForm, PublicThreadPage};

use crate::public;
use crate::router::{self, Route};
use crate::routes::format::relative_time;
use crate::state::alive::Alive;

/// One post and its thread, read anonymously from the instance that hosts it.
#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "a Leptos component receives its props by value"
)]
pub fn PublicThread(host: String, id: String) -> AnyView {
    // A link that names only an id cannot be read: the id is local to an instance. Ask
    // rather than guess, and ask inside the page so the reader keeps the link.
    if host.trim().is_empty() {
        let (typed, set_typed) = signal(String::new());
        let for_submit = id.clone();
        return view! {
            <HostForm
                typed=Signal::derive(move || typed.get())
                on_input=Callback::new(move |value: String| set_typed.set(value))
                on_read=Callback::new(move |()| {
                    let host = typed.get_untracked().trim().to_owned();
                    if host.is_empty() {
                        return;
                    }
                    router::navigate(Route::Thread {
                        host,
                        id: for_submit.clone(),
                    });
                })
            />
        }
        .into_any();
    }

    let alive = Alive::new();
    let (status, set_status) = signal(None::<Status>);
    let (ancestors, set_ancestors) = signal(Vec::<Status>::new());
    let (replies, set_replies) = signal(Vec::<Status>::new());
    let (error, set_error) = signal(None::<String>);
    let (loading, set_loading) = signal(true);

    let for_post = (host.clone(), id.clone());
    Effect::new(move |_| {
        let (host, id) = for_post.clone();
        spawn_local(async move {
            // Two requests, two outcomes: a thread whose context endpoint refuses still has
            // a post worth showing, so the post decides success and the context is best
            // effort.
            match public::status(&host, &id).await {
                Ok(found) => {
                    alive.if_alive(|| set_status.set(Some(found.clone())));
                    if let Ok(context) = public::context(&host, &id).await {
                        alive.if_alive(|| {
                            set_ancestors.set(context.ancestors);
                            set_replies.set(context.descendants);
                        });
                    }
                }
                Err(failure) => {
                    let message = failure.message();
                    alive.if_alive(|| set_error.set(Some(message)));
                }
            }
            alive.if_alive(|| set_loading.set(false));
        });
    });

    let age_of = Callback::new(move |created: String| relative_time(&created));

    view! {
        <PublicThreadPage
            host=host
            status=Signal::derive(move || status.get())
            ancestors=Signal::derive(move || ancestors.get())
            replies=Signal::derive(move || replies.get())
            loading=Signal::derive(move || loading.get())
            error=Signal::derive(move || error.get())
            age_of=age_of
        />
    }
    .into_any()
}
