//! The timeline strip: Home, Feed, Local, Federated — links, because a timeline is an address.

use leptos::prelude::*;
/// A timeline the reader can switch to, as a link.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimelineLink {
    pub label: &'static str,
    pub href: &'static str,
}

/// The four timelines, in the order the tab strip shows them.
pub const TIMELINES: [TimelineLink; 4] = [
    TimelineLink {
        label: "Home",
        href: "/",
    },
    TimelineLink {
        label: "Feed",
        href: "/feed",
    },
    TimelineLink {
        label: "Local",
        href: "/local",
    },
    TimelineLink {
        label: "Federated",
        href: "/federated",
    },
];

/// The tab strip under the composer.
///
/// Links, not a tab signal: each timeline is an address, so a reload keeps the timeline the
/// reader was reading. The server draws every one of them; a reader who turned a timeline
/// off sees it here until the client takes over, which is a smaller surprise than a strip
/// that grows after hydration.
#[component]
pub fn TimelineTabs(#[prop(into)] active: Signal<String>) -> impl IntoView {
    view! {
        <div role="tablist" class="tabs tabs-box mb-2 w-full flex-wrap">
            {TIMELINES
                .iter()
                .map(|timeline| {
                    let href = timeline.href;
                    let label = timeline.label;
                    view! {
                        <a
                            role="tab"
                            class="tab"
                            href=href
                            class:tab-active=move || active.get() == href
                        >
                            {label}
                        </a>
                    }
                })
                .collect::<Vec<_>>()}
        </div>
    }
}
