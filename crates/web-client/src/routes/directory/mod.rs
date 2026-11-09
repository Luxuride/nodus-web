//! People the instance suggests.
//!
//! Read out of the SDK's page store, like every other list: opening this screen again
//! paints what was already fetched, and only a page older than the store's TTL asks the
//! instance for more. The directory is the one list Mastodon pages by offset, so its
//! cursor is a count — the store treats it as an opaque string either way.
//!
//! The screen is `nodus_view::pages::directory`; this file answers what it asks: which scope
//! is showing, the page that scope holds, and where a press on a row goes.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_sdk::page::PageKind;
use nodus_view::pages::directory::DirectoryPage;

use crate::router::{self, Route};
use crate::state::alive::Alive;
use crate::state::bridge::{pages, revision};

/// Local or federated accounts, in the instance's own order of activity.
#[component]
pub fn Directory() -> impl IntoView {
    let revision = revision();
    let (local_only, set_local_only) = signal(true);
    // Starts true: the first thing this screen does is ask, so "empty" before the answer
    // would be a guess.
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);
    let alive = Alive::new();

    let kind = move || PageKind::Directory(local_only.get());
    let held = move || {
        revision.get();
        pages::accounts(&kind())
    };
    let people = move || held().map(|(list, _)| list).unwrap_or_default();
    let cursor = move || held().and_then(|(_, cursor)| cursor);

    let load = move |after: Option<String>| {
        let kind = kind();
        spawn_local(async move {
            match pages::ensure_after(kind, after).await {
                Ok(_) => alive.if_alive(|| set_loading.set(false)),
                Err(message) => alive.if_alive(|| {
                    set_error.set(Some(message));
                    set_loading.set(false);
                }),
            }
        });
    };

    // One effect for both scopes: switching asks the store for the other key, which is
    // either already held or fetched once.
    Effect::new(move |_| {
        let _ = local_only.get();
        load(None);
    });

    let on_scope = Callback::new(move |here: bool| set_local_only.set(here));
    let on_more = Callback::new(move |after: String| load(Some(after)));
    let on_open = Callback::new(move |who: String| router::navigate(Route::Account(who)));

    view! {
        <DirectoryPage
            local_only=local_only.into()
            accounts=Signal::derive(people)
            loading=loading.into()
            error=error.into()
            cursor=Signal::derive(cursor)
            on_scope=on_scope
            on_open=on_open
            on_more=on_more
        />
    }
}
