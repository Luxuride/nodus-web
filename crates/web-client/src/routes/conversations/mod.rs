//! Direct conversations.
//!
//! A conversation is a group of accounts and the last status they sent, which is why the
//! screen shows the last message rather than a message list: opening one is a thread.
//!
//! The screen is `nodus_view::pages::messages`: the heading, the list container, the empty
//! line, the paging control and the error line. What stays here is the store read, the two
//! verbs the instance answers (mark read, mute) and the mute state itself, which the payload
//! does not carry.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_sdk::Command;
use nodus_sdk::page::{PageItems, PageKind};
use nodus_view::pages::messages::MessagesPage;

use crate::routes::format::relative_time;
use crate::state::alive::Alive;
use crate::state::bridge::{actions, dispatch, pages, revision};

/// Mutes or unmutes a conversation, and remembers what was asked for.
///
/// The mute state is not part of the conversation payload, so the screen keeps the list
/// of what it muted; the server's answer replaces the row's last message either way.
fn apply_mute(
    id: String,
    mute_now: bool,
    set_muted: WriteSignal<Vec<String>>,
    set_error: WriteSignal<Option<String>>,
    alive: Alive,
) {
    spawn_local(async move {
        match actions::mute_conversation(&id, mute_now).await {
            Ok(updated) => alive.if_alive(|| {
                dispatch(Command::ConversationsUpserted {
                    items: vec![updated],
                });
                set_muted.update(|held| {
                    held.retain(|known| known != &id);
                    if mute_now {
                        held.push(id.clone());
                    }
                });
            }),
            Err(message) => alive.if_alive(|| set_error.set(Some(message))),
        }
    });
}

/// Marks a conversation read; a failure is ignored on purpose.
fn open_conversation(id: String, alive: Alive) {
    spawn_local(async move {
        if actions::mark_conversation_read(&id).await.is_ok() {
            // The endpoint answers with nothing, so the flag is set on the copy the SDK
            // holds: the server accepted the read, and that is what the row now shows.
            alive.if_alive(|| {
                if let Some((list, _)) = pages::items(&PageKind::Conversations, |items| match items
                {
                    PageItems::Conversations(list) => Some(list),
                    _ => None,
                }) {
                    if let Some(mut conversation) = list.into_iter().find(|item| item.id == id) {
                        conversation.unread = false;
                        dispatch(Command::ConversationsUpserted {
                            items: vec![conversation],
                        });
                    }
                }
            });
        }
    });
}

/// The conversations the account is part of.
#[component]
pub fn Conversations() -> impl IntoView {
    let revision = revision();
    let (muted, set_muted) = signal(Vec::<String>::new());
    let (loading, set_loading) = signal(!pages::held(&PageKind::Conversations));
    let (error, set_error) = signal(None::<String>);
    let alive = Alive::new();

    // The list lives in the SDK: a visit paints it before any request is made, which is
    // what removes the placeholder cards on a revisit.
    let held = move || {
        revision.get();
        pages::items(&PageKind::Conversations, |items| match items {
            PageItems::Conversations(list) => Some(list),
            _ => None,
        })
    };
    let items = move || held().map(|(list, _)| list).unwrap_or_default();
    let cursor = move || held().and_then(|(_, cursor)| cursor);

    let load = move |after: Option<String>| {
        let alive = alive;
        spawn_local(async move {
            let outcome = pages::ensure_after(PageKind::Conversations, after).await;
            match outcome {
                Ok(_) => alive.if_alive(|| set_loading.set(false)),
                Err(message) => alive.if_alive(|| {
                    set_error.set(Some(message));
                    set_loading.set(false);
                }),
            }
        });
    };

    Effect::new(move |_| load(None));
    let on_mute = Callback::new(move |(id, mute_now): (String, bool)| {
        apply_mute(id, mute_now, set_muted, set_error, alive);
    });
    let on_open = Callback::new(move |id: String| open_conversation(id, alive));
    let on_more = Callback::new(move |after: String| load(Some(after)));
    // The row's age is asked for one timestamp at a time, so the page reads the clock
    // through this call and never on its own.
    let age_of = Callback::new(move |created_at: String| relative_time(&created_at));

    view! {
        <MessagesPage
            conversations=Signal::derive(items)
            muted=muted.into()
            loading=loading.into()
            error=error.into()
            cursor=Signal::derive(cursor)
            age_of=age_of
            on_open=on_open
            on_read=on_open
            on_mute=on_mute
            on_more=on_more
        />
    }
}
