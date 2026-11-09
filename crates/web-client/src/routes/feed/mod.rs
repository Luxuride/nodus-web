//! The Feed tab's two lists.
//!
//! Both are recommendations from the same service, differing only in what is
//! recommended: accounts to follow, and posts to read. They live behind one tab bar
//! because they answer one question ("what is missing from my timeline?"), and each
//! keeps its own state so switching tabs never discards a list the other one loaded.

use leptos::prelude::*;

use self::people::People;
use self::posts::Posts;
use crate::routes::home::TimelineHeader;
use crate::state::bridge::revision;

mod actions;
mod item;
mod people;
pub mod posts;
mod reason;
pub(crate) mod share;

/// Which list the Feed tab shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    /// Accounts to follow.
    People,
    /// Posts to read.
    Posts,
}

/// The Feed tab.
#[component]
pub fn Feed() -> impl IntoView {
    // Read once so the counts and any refresh affordance in the header agree with
    // whichever list is rendered below.
    let revision = revision();
    let tab = RwSignal::new(Tab::Posts);

    view! {
        <nodus_view::Screen>
            // The same header as Home — composer and the way between timelines — so the
            // Feed is a sibling destination and not a detour out of the primary flow.
            <TimelineHeader/>

            <div class="mb-5 flex items-baseline justify-between gap-4">
                <nodus_view::PageHeader title="Feed" description="People this instance suggests, and what they are posting."/>
                {move || {
                    revision.get();
                    view! {
                        <div class="tabs tabs-box bg-base-100 p-1">
                            <button
                                class="tab tab-sm"
                                class:tab-active=move || tab.get() == Tab::People
                                on:click=move |_| tab.set(Tab::People)
                            >
                                "People"
                            </button>
                            <button
                                class="tab tab-sm"
                                class:tab-active=move || tab.get() == Tab::Posts
                                on:click=move |_| tab.set(Tab::Posts)
                            >
                                "Posts"
                            </button>
                        </div>
                    }
                }}
            </div>

            // Both lists stay mounted once visited: `Show` with a `when` per tab
            // would rebuild the other list's DOM on every switch, and each list
            // holds its scroll position while it is hidden, not while it is gone.
            <div class:hidden=move || tab.get() != Tab::People>
                <People/>
            </div>
            <div class:hidden=move || tab.get() != Tab::Posts>
                <Posts/>
            </div>
        </nodus_view::Screen>
    }
}
