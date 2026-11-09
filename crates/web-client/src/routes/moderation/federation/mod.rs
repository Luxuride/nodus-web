//! Domains this instance refuses.
//!
//! The outbound half of federation lives under Administration (`relays` and allowed
//! domains), because allowing a domain is a decision about the network, while blocking
//! one is a decision about what the people here have to see.

use leptos::prelude::*;

mod blocked;

use blocked::BlockedList;

#[component]
pub fn Federation() -> impl IntoView {
    view! { <BlockedList/> }
}
