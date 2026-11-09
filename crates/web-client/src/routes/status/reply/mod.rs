//! The reply composer under a thread.
//!
//! Split out because the composer owns state the thread does not care about (the draft,
//! whether a request is in flight) and because it is the part of the thread most likely
//! to grow: media, content warnings and polls all attach here.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::Status;

use crate::state::alive::Alive;
use crate::state::bridge::actions;

/// A text box that publishes a public reply to `parent` and hands the answer back.
#[component]
pub(super) fn ReplyBox(
    parent: String,
    set_replies: WriteSignal<Vec<Status>>,
    set_error: WriteSignal<Option<String>>,
) -> impl IntoView {
    let (draft, set_draft) = signal(String::new());
    let (posting, set_posting) = signal(false);

    let publish = move |()| {
        let text = draft.get_untracked();
        if text.trim().is_empty() {
            return;
        }
        let parent = parent.clone();
        let alive = Alive::new();
        set_posting.set(true);
        set_error.set(None);
        spawn_local(async move {
            match actions::publish(&actions::NewStatus::reply(text, &parent)).await {
                Ok(reply) => {
                    alive.if_alive(|| {
                        set_draft.set(String::new());
                        set_replies.update(|replies| replies.push(reply));
                    });
                }
                Err(message) => alive.if_alive(|| set_error.set(Some(message))),
            }
            alive.if_alive(|| set_posting.set(false));
        });
    };

    view! {
        <nodus_pages::pages::reply_card::ReplyCard
            text=draft.get()
            busy=posting.get()
            on_input=Callback::new(move |text: String| set_draft.set(text))
            on_submit=Callback::new(move |()| publish(()))
        />
    }
}
