//! Hand-written routing: three screens, so no router dependency.
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

use crate::state::oauth::CALLBACK_PATH;

thread_local! {
    /// The current route; created on first read from the address bar.
    static ROUTE: RefCell<Option<RwSignal<Route>>> = const { RefCell::new(None) };
    /// The `popstate` listener, kept alive for the life of the page.
    static POPSTATE: RefCell<Option<WindowListenerHandle>> = const { RefCell::new(None) };
}

/// A screen the app can show.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    /// The recommendation Feed tab.
    Feed,
    /// Instance, sync consent and sign-out.
    Settings,
    /// Where the instance returns the browser after authorization. The query string
    /// carries the code and `state`, so this route is matched regardless of sign-in.
    OauthCallback,
}

impl Route {
    /// The URL path for this route.
    #[must_use]
    pub const fn path(self) -> &'static str {
        match self {
            Self::Feed => "/",
            Self::Settings => "/settings",
            // The OAuth flow registers this path as `redirect_uris`; sharing the
            // constant is what keeps the address bar and the registered URI identical.
            Self::OauthCallback => CALLBACK_PATH,
        }
    }

    /// The route for an address-bar pathname; unknown paths fall back to the feed.
    #[must_use]
    pub fn from_path(path: &str) -> Self {
        let path = path.trim_end_matches('/');
        if path == CALLBACK_PATH {
            return Self::OauthCallback;
        }
        match path {
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
}

/// Navigates to `to`, pushing a history entry so the URL and the back button work.
pub fn navigate(to: Route) {
    if let Some(window) = web_sys::window() {
        if let Ok(history) = window.history() {
            let _ = history.push_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(to.path()));
        }
    }
    route().set(to);
}

/// Click handler for internal links: keeps the SPA and the address bar in step.
///
/// `event` is taken by value because that is what Leptos hands a listener; a
/// borrow would add `&` at every call site and buy nothing.
#[expect(
    clippy::needless_pass_by_value,
    reason = "Leptos listeners receive the event by value"
)]
pub fn link(event: MouseEvent, to: Route) {
    event.prevent_default();
    navigate(to);
}

/// The address bar's pathname, or the empty string when there is no `window`.
fn current_pathname() -> String {
    web_sys::window().map_or_else(String::new, |window| {
        window.location().pathname().unwrap_or_default()
    })
}
