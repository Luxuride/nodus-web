//! Federation, in three lists: what we refuse, what we speak through, what we allow.
//!
//! They live on one screen because they answer one question — "is this instance talking to
//! the right half of the network?" — and a moderator who blocks a domain is usually the
//! same person who wonders why a post did not arrive.

use leptos::prelude::*;

mod blocked;
mod panels;

use blocked::BlockedList;
use panels::Federation as Panels;

#[component]
pub fn Federation() -> impl IntoView {
    view! {
        <div class="flex flex-col gap-4">
            <BlockedList/>
            <Panels/>
        </div>
    }
}
