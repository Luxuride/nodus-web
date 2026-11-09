//! Which timelines the navigation offers.
//!
//! A preference, not a permission: hiding "Local" does not stop the client from being
//! able to read it, it stops the rail from offering it. Nothing here talks to the
//! instance — Mastodon has no API for "which timelines this person reads".

use leptos::prelude::*;
use nodus_pages::pages::settings::feeds::{FeedItem, FeedsCard as SharedFeedsCard};

use crate::state::feeds::{self as prefs, Feed};

/// One switch per feed, each saying what it would hide.
#[component]
pub fn FeedsCard() -> impl IntoView {
    let Some(changed) = prefs::revision() else {
        return view! {
            <nodus_ui::Note>"Preferences are not available."</nodus_ui::Note>
        }
        .into_any();
    };

    let feeds = Signal::derive(move || {
        changed.get();
        Feed::ALL
            .into_iter()
            .map(|feed| FeedItem {
                key: feed.key().to_owned(),
                label: feed.label().to_owned(),
                description: feed.description().to_owned(),
                enabled: prefs::enabled(feed),
                fixed: !feed.optional(),
            })
            .collect()
    });

    view! {
        <SharedFeedsCard
            feeds=feeds
            on_toggle=Some(Callback::new(move |(key, on): (String, bool)| {
                if let Some(feed) = feed_of(&key) {
                    prefs::set(feed, on);
                }
            }))
        />
    }
    .into_any()
}

fn feed_of(key: &str) -> Option<Feed> {
    Feed::ALL.into_iter().find(|feed| feed.key() == key)
}
