//! The rail: every destination, in the order a reader scans them, and the doors only some
//! readers have.
//!
//! A `Destination` is data, not markup, so both paints decide which doors to draw from a
//! session each of them knows about.

use leptos::prelude::*;
use std::sync::Arc;

use super::icons::Icon;
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

/// Whether `path` is this destination or one of its sections.
///
/// `/settings/profile` belongs to Settings, and `/saved/bookmarks` belongs to Saved; the
/// rail marks the section, not only the one address that happens to be its default.
#[must_use]
pub fn within(href: &str, path: &str) -> bool {
    if href == "/" {
        return path == "/";
    }
    if path == href {
        return true;
    }
    for section in ["/settings", "/saved", "/admin", "/moderation"] {
        if let Some(index) = href.find(&format!("{section}/")) {
            let root = &href[..index + section.len()];
            if path == root || path.starts_with(&format!("{root}/")) {
                return true;
            }
        }
    }
    path.starts_with(&format!("{href}/"))
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
                    href: "/",
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
                                                class:menu-active=move || within(href, &active.get())
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

/// One rail entry whose destination is the caller's route, not one of `rail()`'s.
///
/// The address is a real `href`, so a modified click still opens a new tab; only a plain
/// primary click is taken over, and only when the caller has somewhere to send it.
#[component]
pub fn RailEntry(
    #[prop(into)] label: String,
    #[prop(into)] hint: String,
    #[prop(into)] href: String,
    icon: Icon,
    #[prop(into)] active: Signal<bool>,
    #[prop(optional_no_strip)] on_click: Option<Callback<()>>,
) -> impl IntoView {
    view! {
        <li>
            <a
                class="flex w-full items-center gap-2 font-medium"
                class:menu-active=move || active.get()
                href=href
                title=hint
                on:click=move |event| {
                    if event.button() != 0
                        || event.meta_key()
                        || event.ctrl_key()
                        || event.shift_key()
                        || event.alt_key()
                    {
                        return;
                    }
                    event.prevent_default();
                    if let Some(on_click) = on_click {
                        on_click.run(());
                    }
                }
            >
                <Icon icon=icon/>
                <span class="truncate">{label}</span>
            </a>
        </li>
    }
}
