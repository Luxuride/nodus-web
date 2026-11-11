//! The timeline strip: Home, Feed, Local, Federated — links, because a timeline is an address.

use leptos::prelude::*;

/// A timeline the reader can switch to, as a link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimelineLink {
    pub label: String,
    pub href: String,
}

/// The four timelines, in the order the tab strip shows them.
#[must_use]
pub fn timelines() -> Vec<TimelineLink> {
    ["Home", "Feed", "Local", "Federated"]
        .into_iter()
        .zip(["/home", "/home/feed", "/home/local", "/home/federated"])
        .map(|(label, href)| TimelineLink {
            label: label.to_owned(),
            href: href.to_owned(),
        })
        .collect()
}

/// The tab strip under the composer.
///
/// Links, not a tab signal: each timeline is an address, so a reload keeps the timeline the
/// reader was reading. The server draws every one of them; a reader who turned a timeline
/// off sees it here until the client takes over, which is a smaller surprise than a strip
/// that grows after hydration. A client that knows which timelines are enabled passes
/// `links`; one that navigates without a document reload passes `on_navigate`.
#[component]
pub fn TimelineTabs(
    #[prop(into)] active: Signal<String>,
    #[prop(optional)] links: Option<Signal<Vec<TimelineLink>>>,
    #[prop(optional)] on_navigate: Option<Callback<String>>,
) -> impl IntoView {
    let rows = Signal::derive(move || links.map_or_else(timelines, |links| links.get()));

    view! {
        <div role="tablist" class="tabs tabs-box mb-2 w-full flex-wrap">
            <For each=move || rows.get() key=|link| link.href.clone() let:link>
                {
                    let is_active = link.href.clone();
                    let href = link.href.clone();
                    let click = link.href.clone();
                    view! {
                        <a
                            role="tab"
                            class="tab"
                            href=href
                            class:tab-active=move || active.get() == is_active
                            on:click=move |event| {
                                if let Some(navigate) = on_navigate {
                                    event.prevent_default();
                                    navigate.run(click.clone());
                                }
                            }
                        >
                            {link.label}
                        </a>
                    }
                }
            </For>
        </div>
    }
}
