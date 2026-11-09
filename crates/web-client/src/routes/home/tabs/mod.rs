//! The timelines this reader can read, under the composer.
//!
//! The strip itself is shared (`nodus_ui::components::timeline_tabs`) so the server
//! paints the same links the client offers; what stays here is which timelines this
//! reader turned on, and that a click is a client-side navigation.

use leptos::prelude::*;
use nodus_ui::components::timeline_tabs::{TimelineLink, TimelineTabs};

use crate::router::{self, Route};
use crate::state::feeds;

/// The timeline strip.
#[component]
pub fn PrimaryTabs() -> impl IntoView {
    let route = router::route();
    let changed = feeds::revision();

    let links = Signal::derive(move || {
        if let Some(changed) = changed {
            changed.get();
        }
        let mut links = vec![link(Route::Home, "Home")];
        if feeds::enabled(feeds::Feed::Recommended) {
            links.push(link(Route::Feed, "Feed"));
        }
        if feeds::enabled(feeds::Feed::Local) {
            links.push(link(Route::Local, "Local"));
        }
        if feeds::enabled(feeds::Feed::Federated) {
            links.push(link(Route::Federated, "Federated"));
        }
        links
    });

    view! {
        <TimelineTabs
            active=Signal::derive(move || route.get().path())
            links=links
            on_navigate=Callback::new(|path: String| router::navigate(Route::from_path(&path)))
        />
    }
}

/// One timeline as the shared strip wants it.
fn link(route: Route, label: &str) -> TimelineLink {
    TimelineLink {
        label: label.to_owned(),
        href: route.path(),
    }
}
