//! The route vocabulary: one enum, the address-bar path it maps to, and the sections a bare
//! path stands for.
//!
//! Both paints name routes — the client to navigate, the server to know which screen a URL
//! asks for — so the enum lives here, where neither `web_sys` nor axum may be named. Only
//! parsing and formatting are here; pushing history and reading the address bar are
//! `browser::router`'s job.
//!
//! The path is the only source of truth. Unknown paths render the feed, which is what a
//! static host serving `index.html` for every URL will produce anyway.

/// Where the instance sends the browser back.
///
/// The same string is registered as `redirect_uris` and echoed to `/oauth/token`:
/// Mastodon requires all three to match exactly, so it is defined once, here. The route
/// table and `state::oauth` both read it, so the address bar's path and the registered URI
/// cannot drift apart.
pub const CALLBACK_PATH: &str = "/oauth/callback";

/// A screen the app can show.
///
/// Not `Copy`: a thread and a profile carry an id, and an id is a `String` the
/// address bar produced. Everything that compared routes by value now compares them
/// by reference or clones, which is the honest cost of having routes with arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Route {
    /// The recommendation Feed tab.
    Feed,
    /// The home timeline.
    Home,
    /// Posts from accounts on this instance.
    Local,
    /// Posts this instance received from elsewhere.
    Federated,
    /// One status with its ancestors and replies.
    ///
    /// The host is part of the address because an id is local to the instance that issued
    /// it: without the host a link is ambiguous, and a reader who is not signed in cannot
    /// be told where to look. Empty only for a legacy `/status/{id}` link.
    Thread {
        host: String,
        id: String,
    },
    /// One account's profile and statuses.
    Account(String),
    /// Notifications for the signed-in account.
    Messages,
    Notifications,
    /// Search over accounts, statuses and hashtags.
    Search,
    /// People the instance suggests.
    Directory,
    /// Favourites and bookmarks; the shelf is part of the address (`/saved/bookmarks`).
    Saved(String),
    /// Posts waiting to be published.
    Scheduled,
    /// Hand-picked accounts with a timeline each.
    Lists,
    /// The phone's door to everything the bottom bar does not hold.
    More,
    /// Writing a new status.
    Compose,
    /// Editing one of the viewer's own statuses.
    ComposeEdit(String),
    /// Accounts the viewer muted.
    Mutes,
    /// Accounts the viewer blocked.
    Blocks,
    Admin(String),
    Moderation(String),
    /// Settings, with the section in the address (`/settings/privacy`). Empty is the
    /// account section, so `/settings` stays a working link.
    Settings(String),
    /// Where the instance returns the browser after authorization. The query string
    /// carries the code and `state`, so this route is matched regardless of sign-in.
    OauthCallback,
}

impl Route {
    /// The URL path for this route.
    #[must_use]
    pub fn path(&self) -> String {
        let path = self.suffix();
        if BASE.is_empty() {
            return path;
        }
        if path == "/" {
            return BASE.to_owned();
        }
        format!("{BASE}{path}")
    }

    /// The route below the mount point.
    fn suffix(&self) -> String {
        match self {
            Self::Feed => "/feed".to_owned(),
            Self::Home => "/".to_owned(),
            Self::Local => "/local".to_owned(),
            Self::Federated => "/federated".to_owned(),
            Self::Thread { host, id } => {
                if host.is_empty() {
                    format!("/status/{id}")
                } else {
                    format!("/status/{host}/{id}")
                }
            }
            Self::Account(id) => format!("/accounts/{id}"),
            Self::Messages => "/messages".to_owned(),
            Self::Notifications => "/notifications".to_owned(),
            Self::Search => "/search".to_owned(),
            Self::Directory => "/directory".to_owned(),
            Self::Saved(which) => format!("/saved/{which}"),
            Self::Scheduled => "/scheduled".to_owned(),
            Self::Lists => "/lists".to_owned(),
            Self::More => "/more".to_owned(),
            Self::Compose => "/compose".to_owned(),
            Self::ComposeEdit(id) => format!("/compose/{id}"),
            Self::Mutes => "/mutes".to_owned(),
            Self::Blocks => "/blocks".to_owned(),
            Self::Admin(section) => format!("/admin/{section}"),
            Self::Moderation(section) => format!("/moderation/{section}"),
            Self::Settings(section) => format!("/settings/{section}"),
            // The OAuth flow registers this path as `redirect_uris`; sharing the
            // constant is what keeps the address bar and the registered URI identical.
            Self::OauthCallback => CALLBACK_PATH.to_owned(),
        }
    }

    /// The route for an address-bar pathname; unknown paths fall back to the feed.
    ///
    /// A prefix match is how a route with an argument is parsed: the id is whatever
    /// follows, unencoded, because an instance id is a digit string and encoding it
    /// would only make the address bar lie about what the id is.
    #[must_use]
    pub fn from_path(mut path: &str) -> Self {
        // A build mounted under a prefix sees the prefix in the address bar but not in
        // its own routes; stripping it here is what lets one bundle serve both the root
        // and a diagnostic mount without a second copy of every match arm.
        if !BASE.is_empty() {
            path = path.strip_prefix(BASE).unwrap_or(path);
        }
        let path = if path.is_empty() { "/" } else { path };
        let trimmed = path.trim_end_matches('/');
        if trimmed == CALLBACK_PATH {
            return Self::OauthCallback;
        }
        if let Some(rest) = trimmed.strip_prefix("/status/") {
            // `/status/{host}/{id}` is the shareable form; `/status/{id}` stays the
            // in-session one, because an id alone says nothing about where to read it.
            if let Some((host, id)) = rest.split_once('/') {
                if !host.is_empty() && !id.is_empty() {
                    return Self::Thread {
                        host: host.to_owned(),
                        id: id.to_owned(),
                    };
                }
            } else if !rest.is_empty() {
                // Legacy shape: the view resolves the host, or asks for it.
                return Self::Thread {
                    host: String::new(),
                    id: rest.to_owned(),
                };
            }
        }
        // Submenus are addresses, not client state: `/settings/privacy` and
        // `/saved/bookmarks` stay readable, shareable and back-button-friendly.
        if let Some(section) = trimmed.strip_prefix("/settings/") {
            if !section.is_empty() {
                return Self::Settings(section.to_owned());
            }
        }
        if let Some(section) = trimmed.strip_prefix("/moderation/") {
            if !section.is_empty() {
                return Self::Moderation(section.to_owned());
            }
        }
        if let Some(section) = trimmed.strip_prefix("/admin/") {
            if !section.is_empty() {
                return Self::Admin(section.to_owned());
            }
        }
        if let Some(shelf) = trimmed.strip_prefix("/saved/") {
            if !shelf.is_empty() {
                return Self::Saved(shelf.to_owned());
            }
        }
        if let Some(id) = trimmed.strip_prefix("/accounts/") {
            if !id.is_empty() {
                return Self::Account(id.to_owned());
            }
        }
        // After `/accounts/`, so a status id that looks like a path cannot shadow a
        // profile: the longer prefix wins by being tested first.
        if let Some(id) = trimmed.strip_prefix("/compose/") {
            if !id.is_empty() {
                return Self::ComposeEdit(id.to_owned());
            }
        }
        match trimmed {
            "/feed" => Self::Feed,
            "/local" => Self::Local,
            "/federated" => Self::Federated,
            "/messages" => Self::Messages,
            "/notifications" => Self::Notifications,
            "/search" => Self::Search,
            "/directory" => Self::Directory,
            "/saved" => Self::Saved(SHELF_DEFAULT.to_owned()),
            "/scheduled" => Self::Scheduled,
            "/lists" => Self::Lists,
            "/more" => Self::More,
            "/compose" => Self::Compose,
            "/mutes" => Self::Mutes,
            "/blocks" => Self::Blocks,
            "/admin" => Self::Admin(ADMIN_DEFAULT.to_owned()),
            "/moderation" => Self::Moderation(MODERATION_DEFAULT.to_owned()),
            "/settings" => Self::Settings(SECTION_DEFAULT.to_owned()),
            _ => Self::Home,
        }
    }
}

/// Where this bundle is mounted: the root for a release build, a prefix for a diagnostic
/// one (`NODUS_BASE=/debug`). Every route path is composed from it, so an asset or a link
/// cannot point outside the mount by accident.
/// The section `/settings` means when no section is named.
pub const SECTION_DEFAULT: &str = "account";
/// The shelf `/saved` means when no shelf is named.
pub const SHELF_DEFAULT: &str = "favourites";

/// The administration section `/admin` lands on. Declared here rather than in the admin
/// module so a build without the moderation screens can still parse the address.
pub const ADMIN_DEFAULT: &str = "overview";

/// The moderation section `/moderation` lands on: the queue is what a moderator opens.
pub const MODERATION_DEFAULT: &str = "reports";

pub const BASE: &str = match option_env!("NODUS_BASE") {
    Some(base) => base,
    None => "",
};
