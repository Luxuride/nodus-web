//! Hand-written routing: a handful of screens, so no router dependency.
//!
//! `leptos_router` is not a dependency of this crate and three routes do not justify
//! it: the current route is one `RwSignal`, links push a history entry, and a
//! `popstate` listener (the back/forward buttons) re-reads the address bar.
//!
//! [`Route`] and its path arithmetic live in `crate::shared::router`, because both paints
//! name the same screens; this module is the half that needs a `window`. The path is the
//! only source of truth. Unknown paths render the feed, which is what a static host
//! serving `index.html` for every URL will produce anyway.

use std::cell::RefCell;

use leptos::ev::MouseEvent;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

// Named by the staff doors in `app`, which only exist in a moderator build.
#[cfg(feature = "admin")]
pub use crate::shared::router::{ADMIN_DEFAULT, MODERATION_DEFAULT};

pub use crate::shared::router::{Route, SECTION_DEFAULT, SHELF_DEFAULT};

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
    crate::state::epoch::bump();
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
