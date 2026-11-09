//! The feed, drawn by whichever runtime is painting it.
//!
//! The two lists are reads, not route state: when the runtime has them they arrive as
//! cards, and when it does not the same lists draw their own empty branch. The client's
//! impressions and copy verbs ride on the row callback, never on the page.

use std::sync::Arc;

use leptos::prelude::*;
use nodus_shared::{AccountItem, PostItem};

use crate::components::CardRenderer;
use crate::pages::feed::accounts::PeopleList;
use crate::pages::feed::posts::PostList;
use crate::pages::feed::{FeedPage, FeedTab};
use crate::route::Runtime;

#[component]
pub fn FeedScreen(runtime: Arc<dyn Runtime>, card: CardRenderer) -> impl IntoView {
    let _ = (&runtime, &card);
    let tab = RwSignal::new(FeedTab::Posts);
    let people = Signal::derive(Vec::<AccountItem>::new);
    let posts = Signal::derive(Vec::<PostItem>::new);
    let no_error = Signal::derive(|| None::<String>);
    let idle = Signal::derive(|| false);
    let people_row = Callback::new(|_| ().into_any());
    let post_row = Callback::new(|_| ().into_any());
    let people_slot: ChildrenFn = Arc::new(move || {
        view! {
            <PeopleList
                items=people
                error=no_error
                refreshing=idle
                degraded=Signal::derive(|| None::<String>)
                cursor=Signal::derive(|| None::<String>)
                row=people_row
            />
        }
        .into_any()
    });
    let posts_slot: ChildrenFn = Arc::new(move || {
        view! {
            <PostList
                items=posts
                error=no_error
                refreshing=idle
                cursor=Signal::derive(|| None::<String>)
                row=post_row
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
