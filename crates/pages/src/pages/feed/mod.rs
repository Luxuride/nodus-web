//! The feed screen: two lists under one header, one lit at a time.
//!
//! The screen is the shell around two lists the client already owns: people this instance
//! suggests, and posts worth reading. Both stay mounted and are hidden by class, because
//! switching tabs must not throw away the scroll position or an impression that is still
//! being timed; the lit tab is the caller's state, since it is also the screen's address.
//!
//! The header is a slot: the feed is the one screen with a composer above the title, and the
//! composer talks to the SDK, so the client places it and this crate only leaves room.

use leptos::prelude::*;

use crate::chrome::ButtonTabList;
use crate::{PageHeader, Screen};

pub mod accounts;
pub mod posts;
pub mod reason;

/// The two lists the feed shows.
///
/// The caller holds this because the address holds it too; the page only reads which one is
/// lit and reports presses back.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedTab {
    People,
    Posts,
}

/// The feed: a composer slot, a title, two tabs, and the two lists.
///
/// With no `on_tab` the tabs keep their shape and are disabled, which is what the server
/// paints: there is no state to switch there. `people` and `posts` are the lists themselves,
/// built by the client so the same page can hold a store-backed list and a painted one.
#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "a Leptos component receives its props by value"
)]
pub fn FeedPage(
    /// Which list is showing.
    #[prop(into)]
    tab: Signal<FeedTab>,
    /// Switches list. Absent where there is nothing to switch.
    #[prop(optional)]
    on_tab: Option<Callback<FeedTab>>,
    /// What the client puts above the title, usually the composer.
    #[prop(optional_no_strip)]
    header: Option<ChildrenFn>,
    /// The accounts list.
    people: ChildrenFn,
    /// The posts list.
    posts: ChildrenFn,
) -> impl IntoView {
    let header = header.map(|header| header());
    let people = people();
    let posts = posts();

    view! {
        <Screen>
            <div class="flex flex-col gap-2">{header}</div>

            <div class="mb-5 flex flex-wrap items-end justify-between gap-3">
                <PageHeader
                    title="Feed"
                    description="People this instance suggests, and what they are posting."
                />
                <ButtonTabList
                    labels=vec!["People".to_owned(), "Posts".to_owned()]
                    active=Signal::derive(move || {
                        match tab.get() {
                            FeedTab::People => "People".to_owned(),
                            FeedTab::Posts => "Posts".to_owned(),
                        }
                    })
                    on_select=on_tab.map_or_else(
                        || Callback::new(|_: String| ()),
                        |on_tab| {
                            Callback::new(move |label: String| {
                                on_tab.run(if label == "People" {
                                    FeedTab::People
                                } else {
                                    FeedTab::Posts
                                });
                            })
                        },
                    )
                />
            </div>

            <div class:hidden=move || tab.get() != FeedTab::People>{people}</div>
            <div class:hidden=move || tab.get() != FeedTab::Posts>{posts}</div>
        </Screen>
    }
}
