//! The screens both paints know by name.
//!
//! One table, consulted by the client's router and the server's request handler. The point
//! is that a new address is added once: a screen whose title or data source lives in only
//! one of the two paints is how SSR and CSR drift apart.
//!
//! A screen here is *addressable and carries data*. `/accounts/{id}` and `/status/{id}`
//! take a dynamic segment, so they are listed with their prefix and resolved by
//! [`Screen::of_path`].
//!
//! ```
//! use nodus_web::shared::screens::{Screen, ScreenKind};
//!
//! assert_eq!(Screen::of_path("/messages").map(|screen| screen.kind), Some(ScreenKind::Messages));
//! assert_eq!(Screen::of_path("/status/01H").map(|screen| screen.title), Some("Post"));
//! ```

/// What a screen is for, which is what decides the read it needs and how it is titled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenKind {
    /// The signed-in home timeline.
    Home,
    /// Posts this instance suggests.
    Feed,
    /// A public timeline: `local` or everything the instance has received.
    Public { local: bool },
    /// Mentions, boosts and other notifications.
    Notifications,
    /// Direct conversations.
    Messages,
    /// Saved posts, in one of two shelves.
    Saved { bookmarks: bool },
    /// Lists and their timelines.
    Lists,
    /// The phone's directory of every other screen.
    More,
    /// Posts waiting to be published.
    Scheduled,
    /// People this instance publishes in its directory.
    Directory,
    /// A search of accounts, posts and tags.
    Search,
    /// One account and its posts.
    Account,
    /// One status and its context.
    Status,
    /// The moderation areas.
    Moderation,
    /// The administration areas.
    Admin,
    /// Settings, per section.
    Settings,
}

/// One addressable screen.
#[derive(Debug, Clone, Copy)]
pub struct Screen {
    /// The path the paint resolves, without a trailing slash.
    pub path: &'static str,
    /// The browser tab's name, shared so the two paints cannot word it differently.
    pub title: &'static str,
    /// What the screen needs.
    pub kind: ScreenKind,
}

impl Screen {
    /// The screen a path names, dynamic segments included.
    ///
    /// Query strings are the caller's business: `/search?q=x` is `/search`. An unknown path
    /// is not an error here — the client falls back to Home and the server answers 404's
    /// shape, and both of those are the caller's decision, not the table's.
    #[must_use]
    pub fn of_path(path: &str) -> Option<Self> {
        let path = path.split(['?', '#']).next().unwrap_or(path);
        let path = path.strip_suffix('/').filter(|rest| !rest.is_empty()).unwrap_or(path);

        if let Some(screen) = SCREENS.iter().find(|screen| screen.path == path) {
            return Some(*screen);
        }
        if path.starts_with("/status/") {
            return Self::named("/status", ScreenKind::Status);
        }
        if path.starts_with("/accounts/") {
            return Self::named("/accounts", ScreenKind::Account);
        }
        if path.starts_with("/saved/") {
            return Self::named("/saved", ScreenKind::Saved { bookmarks: false });
        }
        None
    }

    fn named(path: &'static str, kind: ScreenKind) -> Option<Self> {
        SCREENS
            .iter()
            .find(|screen| screen.path == path)
            .map(|screen| Self { title: screen.title, kind, ..*screen })
    }

    /// The title to put in `<title>`, with the instance name appended by the caller.
    #[must_use]
    pub fn document_title(&self, instance: &str) -> String {
        format!("{} · {instance}", self.title)
    }
}

/// Every screen, in the order the rail lists them.
pub const SCREENS: &[Screen] = &[
    Screen { path: "/", title: "Home", kind: ScreenKind::Home },
    Screen { path: "/feed", title: "Feed", kind: ScreenKind::Feed },
    Screen { path: "/home", title: "Home", kind: ScreenKind::Home },
    Screen { path: "/local", title: "Local", kind: ScreenKind::Public { local: true } },
    Screen { path: "/federated", title: "Federated", kind: ScreenKind::Public { local: false } },
    Screen { path: "/notifications", title: "Notifications", kind: ScreenKind::Notifications },
    Screen { path: "/messages", title: "Messages", kind: ScreenKind::Messages },
    Screen { path: "/saved", title: "Saved", kind: ScreenKind::Saved { bookmarks: false } },
    Screen { path: "/saved/favourites", title: "Saved", kind: ScreenKind::Saved { bookmarks: false } },
    Screen { path: "/saved/bookmarks", title: "Saved", kind: ScreenKind::Saved { bookmarks: true } },
    Screen { path: "/lists", title: "Lists", kind: ScreenKind::Lists },
    Screen { path: "/more", title: "More", kind: ScreenKind::More },
    Screen { path: "/scheduled", title: "Scheduled", kind: ScreenKind::Scheduled },
    Screen { path: "/directory", title: "People", kind: ScreenKind::Directory },
    Screen { path: "/search", title: "Search", kind: ScreenKind::Search },
    Screen { path: "/status", title: "Post", kind: ScreenKind::Status },
    Screen { path: "/accounts", title: "Account", kind: ScreenKind::Account },
    Screen { path: "/moderation", title: "Moderation", kind: ScreenKind::Moderation },
    Screen { path: "/admin", title: "Administration", kind: ScreenKind::Admin },
    Screen { path: "/settings", title: "Settings", kind: ScreenKind::Settings },
];

#[cfg(test)]
mod tests {
    use super::{Screen, ScreenKind, SCREENS};

    #[test]
    fn every_screen_resolves_to_itself() {
        for screen in SCREENS {
            let resolved = Screen::of_path(screen.path).expect("a listed screen resolves");
            assert_eq!(resolved.kind, screen.kind, "{}", screen.path);
            assert_eq!(resolved.title, screen.title, "{}", screen.path);
        }
    }

    #[test]
    fn dynamic_segments_and_queries_resolve_to_their_screen() {
        assert_eq!(Screen::of_path("/status/01H").map(|s| s.kind), Some(ScreenKind::Status));
        assert_eq!(Screen::of_path("/accounts/7").map(|s| s.kind), Some(ScreenKind::Account));
        assert_eq!(Screen::of_path("/saved/bookmarks").map(|s| s.kind), Some(ScreenKind::Saved { bookmarks: true }));
        assert_eq!(Screen::of_path("/search?q=blobcat").map(|s| s.kind), Some(ScreenKind::Search));
        assert_eq!(Screen::of_path("/local/").map(|s| s.kind), Some(ScreenKind::Public { local: true }));
    }

    #[test]
    fn an_unknown_path_is_not_a_screen() {
        assert!(Screen::of_path("/nope").is_none());
        assert!(Screen::of_path("").is_none());
    }

    #[test]
    fn a_screen_names_its_tab() {
        let screen = Screen::of_path("/messages").expect("messages is a screen");
        assert_eq!(screen.document_title("nodus.luxuride.eu"), "Messages · nodus.luxuride.eu");
    }
}
