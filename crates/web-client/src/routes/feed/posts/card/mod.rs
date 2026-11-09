//! One recommended post, drawn by the shared card.
//!
//! The card itself lives in `nodus_view::pages::feed::posts`; what stays here is the
//! part that is the browser's: reading the recommendation's HTML body as text, and
//! writing the impression when the card leaves the screen (D36).

use leptos::prelude::*;
use nodus_shared::PostItem;

use super::text::plain_text;
use crate::routes::format::relative_time;
use crate::state::bridge::{begin_impression, finish_impression};

/// A post card: who wrote it, what it says, and where to read the original.
#[component]
pub fn PostCard(item: PostItem) -> impl IntoView {
    let body = plain_text(&item.text);
    let age = relative_time(&item.published_at);
    let seen = Callback::new(begin_impression);
    let left = Callback::new(finish_impression);

    view! {
        <nodus_view::pages::feed::posts::PostCard
            item=item
            body=body
            age=age
            on_seen=seen
            on_left=left
        />
    }
}
