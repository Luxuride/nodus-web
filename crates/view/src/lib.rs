//! The parts of the interface that both the browser and the server draw.
//!
//! This crate exists because of one bug that kept coming back: the server painted a page
//! that looked nothing like the one the client painted a moment later, so every load moved
//! the layout under the reader. Two templates cannot be kept in sync; one component tree
//! can. Everything here is pure markup — no `web_sys`, no signals that outlive a render, no
//! router — so it compiles for `wasm32` and for the server, and `leptos::ssr` can turn it
//! into the first paint while the client turns the same tree into the live one.

use leptos::prelude::*;

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
pub fn Rail(#[prop(optional, into)] active: String) -> impl IntoView {
    let active = StoredValue::new(active);
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
                                    let current = active.get_value() == *href;
                                    view! {
                                        <li>
                                            <a
                                                class="flex w-full items-center gap-2 font-medium"
                                                class:menu-active=current
                                                href=href
                                                title=destination.hint
                                            >
                                                <span class="inline-block h-4 w-4 shrink-0 rounded-sm bg-base-300"></span>
                                                <span class="truncate">{destination.label}</span>
                                            </a>
                                        </li>
                                    }
                                })
                                .collect_view()}
                        }
                    })
                    .collect_view()}
            </ul>
        </nav>
    }
}

/// The application's frame: the mobile header, the row, the rail and the column.
///
/// `signed_in` decides whether the rail exists at all, because a signed-out visitor has
/// nowhere to go and the client renders the same nothing.
#[component]
pub fn Frame(
    signed_in: bool,
    #[prop(into)] path: String,
    /// The screen itself. An `AnyView` rather than `Children`, because the server renders a
    /// frame whose content is already a string of HTML and the client renders one whose
    /// content is a live component tree — the same slot takes both.
    inner: AnyView,
) -> impl IntoView {
    let path = StoredValue::new(path);
    view! {
        <div class="min-h-screen bg-base-200 text-base-content">
            <Show when=move || signed_in>
                <header class="sticky top-0 z-30 flex items-center justify-between gap-2 border-b border-base-300 bg-base-100 px-3 py-2 xl:hidden">
                    <a class="font-semibold tracking-tight" href="/home">"Nodus"</a>
                </header>
            </Show>
            <div class="mx-auto flex w-full max-w-6xl flex-col gap-4 px-2 py-4 xl:flex-row">
                <Show when=move || signed_in>
                    <Rail active=path.get_value()/>
                </Show>
                <main class="min-w-0 flex-1 pb-[calc(5.5rem+env(safe-area-inset-bottom))] xl:pb-0">
                    {inner}
                </main>
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rail_has_three_groups_and_nine_destinations() {
        let groups = rail();
        assert_eq!(groups.len(), 3);
        let count: usize = groups.iter().map(|group| group.destinations.len()).sum();
        assert_eq!(count, 9);
    }

    #[test]
    fn every_destination_is_an_absolute_path() {
        for group in rail() {
            for destination in group.destinations {
                assert!(
                    destination.href.starts_with('/'),
                    "{} is not a path",
                    destination.href
                );
            }
        }
    }
}
