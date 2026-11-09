//! Direct conversations.

use std::sync::Arc;

use leptos::prelude::*;
use nodus_state::page::{PageItems, PageKind};

use crate::pages::messages::MessagesPage;
use crate::relative_age;

use super::super::{Action, Runtime};

#[component]
pub fn MessagesScreen(runtime: Arc<dyn Runtime>) -> impl IntoView {
    let revision = runtime.revision();
    let initial = runtime.page(&PageKind::Conversations);
    let loading = RwSignal::new(initial.is_none());
    let error = RwSignal::new(None::<String>);
    let muted = RwSignal::new(Vec::<String>::new());
    let alive = crate::Alive::new();

    let items = {
        let runtime = Arc::clone(&runtime);
        move || {
            revision.get();
            runtime
                .page(&PageKind::Conversations)
                .and_then(|page| match page.items {
                    PageItems::Conversations(items) => Some(items),
                    _ => None,
                })
                .unwrap_or_default()
        }
    };
    let cursor = {
        let runtime = Arc::clone(&runtime);
        move || {
            revision.get();
            runtime
                .page(&PageKind::Conversations)
                .and_then(|page| page.cursor)
        }
    };
    let age_of = Callback::new({
        let runtime = Arc::clone(&runtime);
        move |created: String| relative_age(&created, runtime.now_millis())
    });

    let load: Callback<Option<String>> = Callback::new({
        let runtime = Arc::clone(&runtime);
        move |after: Option<String>| {
            error.set(None);
            loading.set(true);
            let done = Callback::new(move |result: Result<(), String>| {
                alive.if_alive(|| {
                    loading.set(false);
                    if let Err(message) = result {
                        error.set(Some(message));
                    }
                });
            });
            runtime.load_page(PageKind::Conversations, after, done);
        }
    });

    {
        let runtime = Arc::clone(&runtime);
        Effect::new(move |_| {
            if runtime.page(&PageKind::Conversations).is_none() {
                load.run(None);
            }
        });
    }

    let read = Callback::new({
        let runtime = Arc::clone(&runtime);
        move |id: String| {
            let done = Callback::new(|_: Result<crate::route::ActionResult, String>| {});
            runtime.action(Action::ReadConversation(id), done);
        }
    });
    let mute = Callback::new({
        let runtime = Arc::clone(&runtime);
        move |(id, on): (String, bool)| {
            let key = id.clone();
            let done = Callback::new({
                move |result: Result<crate::route::ActionResult, String>| match result {
                    Ok(_) => muted.update(|held| {
                        held.retain(|known| known != &key);
                        if on {
                            held.push(key.clone());
                        }
                    }),
                    Err(message) => error.set(Some(message)),
                }
            });
            runtime.action(Action::MuteConversation { id, on }, done);
        }
    });
    let on_open = read;
    let on_more = Callback::new(move |after: String| load.run(Some(after)));

    view! {
        <MessagesPage
            conversations=Signal::derive(items)
            muted=muted.into()
            loading=loading.into()
            error=error.into()
            cursor=Signal::derive(cursor)
            age_of=age_of
            on_open=on_open
            on_read=read
            on_mute=mute
            on_more=on_more
        />
    }
}
