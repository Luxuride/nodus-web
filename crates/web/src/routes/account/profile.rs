//! How the client paints the shared profile page.
//!
//! The page itself lives in `nodus-view`: the header, the verbs' boxes, the status list and
//! the order of the three. What only the client can supply is the card — which has verbs, a
//! media viewer and a router step — and the clock its age comes from.

use std::sync::Arc;

use leptos::prelude::*;
use nodus_instance::model::Status;
use nodus_view::components::CardRenderer;

use crate::routes::status::StatusCard;

/// The card the profile's list paints: the live one, not the server's inert one.
pub(super) fn card() -> CardRenderer {
    Arc::new(|status: Status| view! { <StatusCard status=status/> }.into_any())
}

/// How old a post is, from the browser's clock.
pub(super) fn age(created_at: &str) -> String {
    crate::routes::format::relative_time(created_at)
}
