//! The block every primary timeline opens with: write, then switch between timelines.

use leptos::prelude::*;
use nodus_client_sdk::Timeline;

use crate::routes::compose::inline::InlineComposer;
use crate::routes::home::PrimaryTabs;

/// The composer and the timeline switch, in that order, above the posts.
///
/// One block used by all four primary timelines, so a reader who arrives on Local or
/// Federated still has the way back and the way to write. The composer's own destination is
/// always Home: a post is published to the account, and Home is where its author will look
/// for it, so a refresh after publishing belongs there whatever screen the box sits on.
#[component]
pub fn TimelineHeader() -> impl IntoView {
    let (_reload, set_reload) = signal(0_u32);
    let publish_to = StoredValue::new(Timeline::Home);

    view! {
        <InlineComposer which=Some(publish_to) set_reload=Some(set_reload)/>
        <PrimaryTabs/>
    }
}
