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
    /// The home timeline, and the public ones beside it.
    Home,
    /// One status with its ancestors and replies.
    Thread(String),
    /// One account's profile and statuses.
    Account(String),
    /// Notifications for the signed-in account.
    Notifications,
    /// Search over accounts, statuses and hashtags.
    Search,
    /// Favourites and bookmarks.
    Saved,
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
    /// Instance, sync consent and sign-out.
    Settings,
    /// Where the instance returns the browser after authorization. The query string
    /// carries the code and `state`, so this route is matched regardless of sign-in.
    OauthCallback,
}

impl Route {
    /// The URL path for this route.
    #[must_use]
    pub fn path(&self) -> String {
        match self {
            Self::Feed => "/".to_owned(),
            Self::Home => "/home".to_owned(),
            Self::Thread(id) => format!("/status/{id}"),
            Self::Account(id) => format!("/accounts/{id}"),
            Self::Notifications => "/notifications".to_owned(),
            Self::Search => "/search".to_owned(),
            Self::Saved => "/saved".to_owned(),
            Self::Lists => "/lists".to_owned(),
            Self::Compose => "/compose".to_owned(),
            Self::ComposeEdit(id) => format!("/compose/{id}"),
            Self::Mutes => "/mutes".to_owned(),
            Self::Blocks => "/blocks".to_owned(),
            Self::Settings => "/settings".to_owned(),
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
    pub fn from_path(path: &str) -> Self {
        let trimmed = path.trim_end_matches('/');
        if trimmed == CALLBACK_PATH {
            return Self::OauthCallback;
        }
        if let Some(id) = trimmed.strip_prefix("/status/") {
            if !id.is_empty() {
                return Self::Thread(id.to_owned());
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
            "/notifications" => Self::Notifications,
            "/search" => Self::Search,
            "/saved" => Self::Saved,
            "/lists" => Self::Lists,
            "/compose" => Self::Compose,
            "/mutes" => Self::Mutes,
            "/blocks" => Self::Blocks,
            "/settings" => Self::Settings,
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
    }
    current.set(to);
}

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
