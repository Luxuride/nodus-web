//! Accounts to follow: the original Feed tab body.
//!
//! The list is shared; this file owns where its rows come from and where its verbs go.
//! Each action is either a state change (dismiss) or a single instance write (follow,
//! unfollow), both funnelled through [`crate::state::bridge`].
//!
//! Four states reach the list: empty, error, refreshing (the list stays visible, dimmed)
//! and degraded (one muted line, because a weaker ranking is not a failure).

use leptos::prelude::*;
use nodus_shared::AccountItem;
use nodus_view::pages::feed::accounts::PeopleList;

use crate::router::{Route, navigate};
use crate::routes::feed::item::FeedItem;
use crate::routes::feed::reason::degraded_line;
use crate::state::bridge::{feed_ui, reload_feed, revision, with_sdk};

/// The list of recommended accounts.
#[component]
pub fn People() -> impl IntoView {
    let revision = revision();
    let ui = feed_ui();

    let items = Signal::derive(move || {
        revision.get();
        with_sdk(|sdk| {
            sdk.state()
                .feed
                .with(|feed| feed.visible().cloned().collect::<Vec<_>>())
        })
    });
    let degraded = Signal::derive(move || {
        revision.get();
        with_sdk(|sdk| sdk.state().feed.get().degraded)
    });
    let next_cursor = Signal::derive(move || {
        revision.get();
        with_sdk(|sdk| sdk.state().feed.get().next_cursor)
    });
    let error = Signal::derive(move || ui.error.get());
    let refreshing = Signal::derive(move || ui.refreshing.get());
    let degraded_text = Signal::derive(move || degraded.get().first().copied().map(degraded_line));

    view! {
        <PeopleList
            items=items
            error=error
            refreshing=refreshing
            degraded=degraded_text
            cursor=next_cursor
            // The follow flag is part of the key: undoing a follow flips it in
            // place, and remounting is what makes the card show "Follow" again
            // without a full page reload.
            row=Callback::new(move |item: AccountItem| view! { <FeedItem item/> }.into_any())
            on_refresh=Callback::new(move |()| reload_feed(None))
            on_more=Callback::new(move |cursor: String| reload_feed(Some(cursor)))
            on_settings=Some(Callback::new(move |()| {
                navigate(Route::Settings(crate::router::SECTION_DEFAULT.to_owned()));
            }))
        />
    }
}
