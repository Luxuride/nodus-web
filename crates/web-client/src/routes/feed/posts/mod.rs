//! Recommended posts: the second half of the Feed tab.
//!
//! Renders `PostFeedState` and nothing else. The text arrives as the instance's
//! HTML; until the sanitizer from `web/docs/04-ui.md` exists in this crate, it is
//! reduced to plain text in [`text`] rather than injected with `inner_html` — a
//! `dangerously_set_inner_html` call is exactly the XSS surface the token-in-the-
//! browser design cannot afford.

use leptos::prelude::*;
use nodus_shared::PostItem;
use nodus_view::pages::feed::posts::PostList;

use crate::state::bridge::{load_posts_once, posts_ui, reload_posts, revision, with_sdk};

mod card;
pub mod text;

pub use self::text::plain_text;

use self::card::PostCard;

/// The list of recommended posts.
#[component]
pub fn Posts() -> impl IntoView {
    load_posts_once();

    let revision = revision();
    let ui = posts_ui();

    let items = Signal::derive(move || {
        revision.get();
        with_sdk(|sdk| sdk.state().posts.get().items)
    });
    let next_cursor = Signal::derive(move || {
        revision.get();
        with_sdk(|sdk| sdk.state().posts.get().next_cursor)
    });

    view! {
        <PostList
            items=items
            error=Signal::derive(move || ui.error.get())
            refreshing=Signal::derive(move || ui.refreshing.get())
            cursor=next_cursor
            row=Callback::new(move |item: PostItem| view! { <PostCard item/> }.into_any())
            on_refresh=Callback::new(move |()| reload_posts(None))
            on_more=Callback::new(move |cursor: String| reload_posts(Some(cursor)))
        />
    }
}
