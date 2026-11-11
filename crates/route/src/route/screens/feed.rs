//! The feed, drawn by whichever runtime is painting it.
//!
//! The list is a read, not route state: when the runtime has it the cards arrive, and
//! when it does not the same list draws its own empty branch. The client's impressions
//! and copy verbs ride on the row callback, never on the page.

use std::sync::Arc;

use leptos::prelude::*;

use crate::components::CardRenderer;
use crate::pages::feed::FeedPage;
use crate::pages::feed::posts::PostList;
use crate::route::{FeedKind, Runtime};

#[component]
pub fn FeedScreen(
    runtime: Arc<dyn Runtime>,
    card: CardRenderer,
    #[prop(optional_no_strip)] header: Option<ChildrenFn>,
) -> impl IntoView {
    let _ = &card;
    let loaded_posts = RwSignal::new(false);

    let posts = Signal::derive({
        let runtime = Arc::clone(&runtime);
        move || runtime.feed_posts()
    });

    let load = Callback::new({
        let runtime = Arc::clone(&runtime);
        move |cursor: Option<String>| {
            let runtime = Arc::clone(&runtime);
            let done = Callback::new(move |_result: Result<(), String>| ());
            runtime.load_feed(FeedKind::Posts, cursor, done);
        }
    });
    let auto = runtime.infinite_more(
        Signal::derive(move || posts.get().cursor.is_some() && !posts.get().refreshing),
        Callback::new(move |()| {
            if let Some(cursor) = posts.get_untracked().cursor {
                load.run(Some(cursor));
            }
        }),
    );

    Effect::new({
        let runtime = Arc::clone(&runtime);
        move |_| {
            if !loaded_posts.get_untracked() && runtime.feed_posts().items.is_empty() {
                loaded_posts.set(true);
                load.run(None);
            }
        }
    });

    let fallback = Callback::new(|_| {
        view! { <crate::Note>"This row needs the app."</crate::Note> }.into_any()
    });
    let row = runtime.feed_posts_row().unwrap_or(fallback);

    let rows: ChildrenFn = Arc::new(move || {
        view! {
            <PostList
                items=Signal::derive(move || posts.get().items.clone())
                error=Signal::derive(move || posts.get().error.clone())
                refreshing=Signal::derive(move || posts.get().refreshing)
                cursor=Signal::derive(move || posts.get().cursor.clone())
                row=row
                on_refresh=Callback::new(move |()| load.run(None))
                on_more=Callback::new(move |cursor: String| load.run(Some(cursor)))
                auto_more=auto.clone()
                show_states=false
            />
        }
        .into_any()
    });

    view! {
        <FeedPage
            title="Feed"
            description="People this instance suggests, and what they are posting."
            empty_note="Suggested posts arrive after the recommender has fetched some timelines."
            header=header
            error=Signal::derive(move || posts.get().error.clone())
            loading=Signal::derive(move || posts.get().refreshing)
            has_items=Signal::derive(move || !posts.get().items.is_empty())
        >
            {rows()}
        </FeedPage>
    }
}
