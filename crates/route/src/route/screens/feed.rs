//! The feed, drawn by whichever runtime is painting it.
//!
//! The two lists are reads, not route state: when the runtime has them they arrive as
//! cards, and when it does not the same lists draw their own empty branch. The client's
//! impressions and copy verbs ride on the row callback, never on the page.

use std::sync::Arc;

use leptos::prelude::*;

use crate::components::CardRenderer;
use crate::pages::feed::accounts::PeopleList;
use crate::pages::feed::posts::PostList;
use crate::pages::feed::{FeedPage, FeedTab};
use crate::route::{FeedKind, Runtime};

#[component]
pub fn FeedScreen(runtime: Arc<dyn Runtime>, card: CardRenderer) -> impl IntoView {
    let _ = &card;
    let tab = RwSignal::new(FeedTab::Posts);
    let loaded_people = RwSignal::new(false);
    let loaded_posts = RwSignal::new(false);

    let people = Signal::derive({
        let runtime = Arc::clone(&runtime);
        move || runtime.feed_people()
    });
    let posts = Signal::derive({
        let runtime = Arc::clone(&runtime);
        move || runtime.feed_posts()
    });

    let load = Callback::new({
        let runtime = Arc::clone(&runtime);
        move |(kind, cursor): (FeedKind, Option<String>)| {
            let runtime = Arc::clone(&runtime);
            let done = Callback::new(move |_result: Result<(), String>| ());
            runtime.load_feed(kind, cursor, done);
        }
    });
    Effect::new({
        let runtime = Arc::clone(&runtime);
        move |_| {
            if !loaded_people.get_untracked() && runtime.feed_people().items.is_empty() {
                loaded_people.set(true);
                load.run((FeedKind::People, None));
            }
        }
    });
    Effect::new({
        let runtime = Arc::clone(&runtime);
        move |_| {
            if !loaded_posts.get_untracked() && runtime.feed_posts().items.is_empty() {
                loaded_posts.set(true);
                load.run((FeedKind::Posts, None));
            }
        }
    });

    let fallback = Callback::new(|_| {
        view! { <crate::Note>"This row needs the app."</crate::Note> }.into_any()
    });
    let people_row = runtime.feed_people_row().unwrap_or(fallback);
    let post_fallback = Callback::new(|_| {
        view! { <crate::Note>"This row needs the app."</crate::Note> }.into_any()
    });
    let posts_row = runtime.feed_posts_row().unwrap_or(post_fallback);

    let people_slot: ChildrenFn = Arc::new(move || {
        view! {
            <PeopleList
                items=Signal::derive(move || people.get().items.clone())
                error=Signal::derive(move || people.get().error.clone())
                refreshing=Signal::derive(move || people.get().refreshing)
                degraded=Signal::derive(move || people.get().degraded.clone())
                cursor=Signal::derive(move || people.get().cursor.clone())
                row=people_row
                on_refresh=Callback::new({
                    let load = load;
                    move |()| load.run((FeedKind::People, None))
                })
                on_more=Callback::new({
                    let load = load;
                    move |cursor: String| load.run((FeedKind::People, Some(cursor)))
                })
                on_settings=runtime.feed_settings()
            />
        }
        .into_any()
    });
    let posts_slot: ChildrenFn = Arc::new(move || {
        view! {
            <PostList
                items=Signal::derive(move || posts.get().items.clone())
                error=Signal::derive(move || posts.get().error.clone())
                refreshing=Signal::derive(move || posts.get().refreshing)
                cursor=Signal::derive(move || posts.get().cursor.clone())
                row=posts_row
                on_refresh=Callback::new({
                    let load = load;
                    move |()| load.run((FeedKind::Posts, None))
                })
                on_more=Callback::new({
                    let load = load;
                    move |cursor: String| load.run((FeedKind::Posts, Some(cursor)))
                })
            />
        }
        .into_any()
    });

    view! {
        <FeedPage
            tab=Signal::derive(move || tab.get())
            on_tab=Callback::new(move |next: FeedTab| tab.set(next))
            people=people_slot
            posts=posts_slot
        />
    }
}
