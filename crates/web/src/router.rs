//! Hand-written routing: a handful of screens, so no router dependency.
//!
//! `leptos_router` is not a dependency of this crate and three routes do not justify
//! it: the current route is one `RwSignal`, links push a history entry, and a
//! `popstate` listener (the back/forward buttons) re-reads the address bar.
//!
//! The path is the only source of truth. Unknown paths render the feed, which is
//! what a static host serving `index.html` for every URL will produce anyway.

use std::cell::RefCell;

use leptos::ev::MouseEvent;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::state::oauth::CALLBACK_PATH;

/// The click listener's closure type, named once so the `thread_local!` stays readable.
type LinkHandler = wasm_bindgen::closure::Closure<dyn FnMut(web_sys::MouseEvent)>;

thread_local! {
    /// The current route; created on first read from the address bar.
    static ROUTE: RefCell<Option<RwSignal<Route>>> = const { RefCell::new(None) };
    /// The `popstate` listener, kept alive for the life of the page.
    static POPSTATE: RefCell<Option<WindowListenerHandle>> = const { RefCell::new(None) };
    /// The delegated link listener, kept alive for the life of the page.
    static LINKS: RefCell<Option<LinkHandler>> = const { RefCell::new(None) };
}

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
            Self::Feed => "/".to_owned(),
            Self::Home => "/home".to_owned(),
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
            "/home" => Self::Home,
            "/local" => Self::Local,
            "/federated" => Self::Federated,
            "/messages" => Self::Messages,
            "/notifications" => Self::Notifications,
            "/search" => Self::Search,
            "/directory" => Self::Directory,
            "/saved" => Self::Saved(SHELF_DEFAULT.to_owned()),
            "/scheduled" => Self::Scheduled,
            "/lists" => Self::Lists,
            "/compose" => Self::Compose,
            "/mutes" => Self::Mutes,
            "/blocks" => Self::Blocks,
            "/admin" => Self::Admin(ADMIN_DEFAULT.to_owned()),
            "/moderation" => Self::Moderation(MODERATION_DEFAULT.to_owned()),
            "/settings" => Self::Settings(SECTION_DEFAULT.to_owned()),
            _ => Self::Feed,
        }
    }
}

/// The current route, initialised from `window.location.pathname`.
pub fn route() -> RwSignal<Route> {
    ROUTE.with_borrow_mut(|slot| {
        *slot.get_or_insert_with(|| RwSignal::new(Route::from_path(&current_pathname())))
    })
}

/// Starts following the back/forward buttons. Idempotent.
pub fn install() {
    POPSTATE.with_borrow_mut(|slot| {
        if slot.is_some() {
            return;
        }
        *slot = Some(window_event_listener_untyped("popstate", |_| {
            route().set(Route::from_path(&current_pathname()));
        }));
    });
    install_links();
}

/// Intercepts clicks on any in-app anchor, wherever it was written.
///
/// One delegated listener rather than an `on:click` on every link: a link rendered by a
/// card, a profile row or the composer behaves the same, and a new screen cannot forget
/// the handler and silently fall back to a full document load — which threw away SDK
/// state and looked like the server had rendered the page.
fn install_links() {
    LINKS.with_borrow_mut(|slot| {
        if slot.is_some() {
            return;
        }
        let handler = LinkHandler::new(|event: web_sys::MouseEvent| {
            if event.default_prevented()
                || event.button() != 0
                || event.meta_key()
                || event.ctrl_key()
                || event.shift_key()
                || event.alt_key()
            {
                return;
            }
            let Some(target) = event.target() else {
                return;
            };
            let Ok(element) = target.dyn_into::<web_sys::Element>() else {
                return;
            };
            let Ok(Some(anchor)) = element.closest("a") else {
                return;
            };
            let Some(href) = anchor.get_attribute("href") else {
                return;
            };
            let path = href.split(['?', '#']).next().unwrap_or_default();
            if !is_internal(path) {
                return;
            }
            event.prevent_default();
            navigate(Route::from_path(path));
        });
        if let Some(document) = web_sys::window().and_then(|window| window.document()) {
            let callback: &js_sys::Function = wasm_bindgen::JsCast::unchecked_ref(handler.as_ref());
            let _ = document.add_event_listener_with_callback("click", callback);
        }
        *slot = Some(handler);
    });
}

/// Whether `path` is a screen this app renders.
///
/// The shared-post page (`/s/…`) is server-rendered and the API lives under `/api`; both
/// must be left to the browser, or the app would try to render a page it does not have.
fn is_internal(path: &str) -> bool {
    path.starts_with('/')
        && !path.starts_with("//")
        && !path.starts_with("/api")
        && !path.starts_with("/s/")
        && !path.starts_with("/healthz")
        && !path.starts_with("/readyz")
}

/// Navigates to `to`, pushing a history entry so the URL and the back button work.
///
/// Re-navigating to the screen already on display is a no-op. Without this, clicking the
/// current tab stacks a second identical history entry, and the Back button then appears
/// to do nothing — it pops to the same URL, and the same screen renders again.
pub fn navigate(to: Route) {
    let current = route();
    let path = to.path();
    let unchanged = current.get_untracked() == to
        && web_sys::window()
            .and_then(|window| window.location().pathname().ok())
            .is_some_and(|here| here == path);
    if unchanged {
        return;
    }
    if let Some(window) = web_sys::window() {
        if let Ok(history) = window.history() {
            // `path()` builds the string, so it has to outlive the call that borrows it.
            let _ = history.push_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(&path));
        }
        // A new screen starts at its top: keeping the previous offset left the reader
        // halfway down a page they had not seen yet.
        window.scroll_to_with_x_and_y(0.0, 0.0);
    }
    current.set(to);
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

/// Click handler for internal links: keeps the SPA and the address bar in step.
///
/// `event` is taken by value because that is what Leptos hands a listener; a
/// borrow would add `&` at every call site and buy nothing.
///
/// A modified or non-primary click is left alone. The `href` is real, so
/// <kbd>ctrl</kbd>/<kbd>cmd</kbd>-click, middle-click and "open in new tab" must do what
/// the browser does with it — swallowing those is how a single-page app quietly stops
/// supporting the one thing people do to links they want to keep.
#[expect(
    clippy::needless_pass_by_value,
    reason = "Leptos listeners receive the event by value"
)]
pub fn link(event: MouseEvent, to: Route) {
    if event.button() != 0
        || event.meta_key()
        || event.ctrl_key()
        || event.shift_key()
        || event.alt_key()
    {
        return;
    }
    event.prevent_default();
    navigate(to);
}

/// The address bar's pathname, or the empty string when there is no `window`.
fn current_pathname() -> String {
    web_sys::window().map_or_else(String::new, |window| {
        window.location().pathname().unwrap_or_default()
    })
}

/// One value from the address bar's query string.
///
/// Read once by a screen that can be arrived at with a question already asked — a hashtag
/// chip links to `/search?q=%23rust`, and an empty search box would throw the question
/// away. Percent escapes are decoded for `%`, `#`, `@` and spaces, which is what a chip and
/// a keyboard produce; anything else is left as written rather than guessed at.
#[must_use]
pub fn query_param(name: &str) -> Option<String> {
    let search = web_sys::window()?.location().search().ok()?;
    let pairs = search.strip_prefix('?')?;
    for pair in pairs.split('&') {
        let (key, value) = pair.split_once('=')?;
        if key == name {
            return Some(decode_query(value));
        }
    }
    None
}

/// Decodes the escapes a link built in this app produces.
fn decode_query(value: &str) -> String {
    let mut decoded = String::with_capacity(value.len());
    let mut rest = value;
    while let Some(index) = rest.find('%') {
        decoded.push_str(&rest[..index]);
        let escape = rest
            .get(index + 1..index + 3)
            .and_then(|hex| u8::from_str_radix(hex, 16).ok());
        if let Some(character) = escape.and_then(|byte| char::from_u32(u32::from(byte))) {
            decoded.push(character);
            rest = &rest[index + 3..];
        } else {
            decoded.push('%');
            rest = &rest[index + 1..];
        }
    }
    decoded.push_str(rest);
    decoded.replace('+', " ")
}
