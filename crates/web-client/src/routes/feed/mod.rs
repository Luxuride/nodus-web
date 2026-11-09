//! The Feed tab's two lists.
//!
//! Both are recommendations from the same service, differing only in what is
//! recommended: accounts to follow, and posts to read. They live behind one tab bar
//! because they answer one question ("what is missing from my timeline?"), and each
//! keeps its own state so switching tabs never discards a list the other one loaded.

use std::sync::Arc;

use leptos::prelude::*;
use nodus_view::pages::feed::{FeedPage, FeedTab};

use self::people::People;
use self::posts::Posts;
use crate::routes::home::TimelineHeader;

mod actions;
mod item;
mod people;
pub mod posts;
mod reason;
pub(crate) mod share;

/// The Feed tab.
#[component]
pub fn Feed() -> impl IntoView {
    let tab = RwSignal::new(FeedTab::Posts);
    let header: ChildrenFn = Arc::new(|| view! { <TimelineHeader/> }.into_any());
    let people: ChildrenFn = Arc::new(|| view! { <People/> }.into_any());
    let posts: ChildrenFn = Arc::new(|| view! { <Posts/> }.into_any());

    view! {
        <FeedPage
            tab=Signal::derive(move || tab.get())
            on_tab=Callback::new(move |next: FeedTab| tab.set(next))
            header=header
            people=people
            posts=posts
        />
    }
}
