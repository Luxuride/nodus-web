//! The parts of the interface that both the browser and the server draw.
//!
//! This crate exists because of one bug that kept coming back: the server painted a page
//! that looked nothing like the one the client painted a moment later, so every load moved
//! the layout under the reader. Two templates cannot be kept in sync; one component tree
//! can. Everything here is pure markup — no `web_sys`, no signals that outlive a render, no
//! router — so it compiles for `wasm32` and for the server, and `leptos::ssr` can turn it
//! into the first paint while the client turns the same tree into the live one.

pub mod icons;

use icons::Icon;
use std::sync::Arc;
use leptos::prelude::*;

/// The drawing for a destination, by name.
///
/// The label is the key rather than a field on `Destination`, because the label is what a
/// reader sees and what the list is ordered by; a second field would be one more thing to
/// keep in step for no gain.
#[must_use]
pub fn icon_of(label: &str) -> Icon {
    match label {
        "Home" => Icon::Home,
        "Feed" => Icon::Sparkle,
        "Local" | "People" => Icon::Users,
        "Federated" => Icon::Globe,
        "Messages" => Icon::Mail,
        "Notifications" => Icon::Bell,
        "Search" => Icon::Search,
        "Saved" => Icon::Bookmark,
        "Scheduled" => Icon::Clock,
        "Lists" => Icon::List,
        "Compose" => Icon::Pen,
        "Moderation" | "Administration" => Icon::Shield,
        _ => Icon::Settings,
    }
}

/// Doors only some readers have, as a function so the same value can be drawn by both
/// paints: the server closes over server markup, the browser closes over live components.
pub type ExtraDoors = Arc<dyn Fn() -> AnyView + Send + Sync>;

/// One destination in the rail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Destination {
    /// What the reader sees.
    pub label: &'static str,
    /// Where it goes.
    pub href: &'static str,
    /// One sentence, shown as the hover title.
    pub hint: &'static str,
}

/// A named group of destinations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group {
    /// The section heading, hidden on narrow screens.
    pub title: &'static str,
    /// Its destinations, in order.
    pub destinations: &'static [Destination],
}

/// The rail the client shows, in its order.
#[must_use]
pub fn rail() -> Vec<Group> {
    vec![
        Group {
            title: "Read",
            destinations: &[
                Destination {
                    label: "Home",
                    href: "/home",
                    hint: "Your timeline, the feed, and the instance's",
                },
                Destination {
                    label: "Messages",
                    href: "/messages",
                    hint: "Direct conversations",
                },
                Destination {
                    label: "Notifications",
                    href: "/notifications",
                    hint: "Mentions, boosts, favourites and follow requests",
                },
            ],
        },
        Group {
            title: "Find",
            destinations: &[
                Destination {
                    label: "Search",
                    href: "/search",
                    hint: "Accounts, posts and hashtags, including remote ones",
                },
                Destination {
                    label: "People",
                    href: "/directory",
                    hint: "Accounts this instance suggests",
                },
                Destination {
                    label: "Saved",
                    href: "/saved/favourites",
                    hint: "Your favourites and bookmarks",
                },
            ],
        },
        Group {
            title: "Library",
            destinations: &[
                Destination {
                    label: "Scheduled",
                    href: "/scheduled",
                    hint: "Posts waiting to be published",
                },
                Destination {
                    label: "Lists",
                    href: "/lists",
                    hint: "Hand-picked accounts with a timeline each",
                },
                Destination {
                    label: "Settings",
                    href: "/settings/account",
                    hint: "Profile, tags, filters, sync and this instance",
                },
            ],
        },
    ]
}

/// The rail.
///
/// The icon box is drawn at the icon's size rather than as an SVG: `view` deliberately has
/// no icon set, and the client swaps in the real paths when it takes over — the box is what
/// keeps the column from moving.
#[component]
pub fn Rail(
    /// The address being shown, so the matching destination is marked.
    #[prop(into)]
    active: Signal<String>,
    /// Doors that only some readers have: the moderation and administration ones.
    extra: Option<ExtraDoors>,
) -> impl IntoView {
    view! {
        <nav class="hidden shrink-0 xl:block xl:w-44">
            <ul class="menu w-full flex-row flex-wrap gap-1 rounded-box bg-base-100 p-2 shadow-sm xl:flex-col xl:flex-nowrap">
                {rail()
                    .into_iter()
                    .map(|group| {
                        view! {
                            <li class="menu-title hidden xl:block">{group.title}</li>
                            {group
                                .destinations
                                .iter()
                                .map(|destination| {
                                    let href = destination.href;
                                    view! {
                                        <li>
                                            <a
                                                class="flex w-full items-center gap-2 font-medium"
                                                class:menu-active=move || active.get() == href
                                                href=href
                                                title=destination.hint
                                            >
                                                <Icon icon=icon_of(destination.label)/>
                                                <span class="truncate">{destination.label}</span>
                                            </a>
                                        </li>
                                    }
                                })
                                .collect_view()}
                        }
                    })
                    .collect_view()}
                {move || extra.as_ref().map(|doors| doors())}
            </ul>
        </nav>
    }
}

#[component]
pub fn Frame(
    /// Whether the reader has a session. The rail is navigation, so it is only there for
    /// someone who can use it.
    #[prop(into)]
    signed_in: Signal<bool>,
    /// The address being shown.
    #[prop(into)]
    path: Signal<String>,
    /// The screen itself. A function, not a value, so a route change repaints the content
    /// without rebuilding the frame around it (which would drop the screen's state).
    inner: ChildrenFn,
    /// Doors only some readers have.
    extra: Option<ExtraDoors>,
) -> impl IntoView {
    let inner = StoredValue::new(inner);
    view! {
        <div class="min-h-screen bg-base-200 text-base-content">
            <Show when=move || signed_in.get()>
                <header class="sticky top-0 z-30 flex items-center justify-between gap-2 border-b border-base-300 bg-base-100 px-3 py-2 xl:hidden">
                    <a class="font-semibold tracking-tight" href="/home">"Nodus"</a>
                </header>
            </Show>
            <div class="mx-auto flex w-full max-w-6xl flex-col gap-4 px-2 py-4 xl:flex-row">
                <Show when=move || signed_in.get()>
                    <Rail active=path extra=extra.clone()/>
                </Show>
                <main class="min-w-0 flex-1 pb-[calc(5.5rem+env(safe-area-inset-bottom))] xl:pb-0">
                    {move || inner.with_value(|inner| inner())}
                </main>
            </div>
        </div>
    }
}
