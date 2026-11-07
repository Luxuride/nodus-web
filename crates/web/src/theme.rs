//! Dark mode, decided by the operating system.
//!
//! daisyUI pairs `data-theme="light"` / `data-theme="dark"` with semantic colour
//! variables, so following the system is one attribute on the document element plus a
//! listener for when the operating system changes its mind mid-session.
//!
//! The first assignment happens in `index.html` before this module runs: a wasm module
//! arrives after the stylesheet, so waiting for it would paint a light frame first — the
//! one flash this feature must not introduce.

use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;

/// The media query that answers "does the operating system want dark".
const QUERY: &str = "(prefers-color-scheme: dark)";

/// Applies the system preference now and whenever it changes.
pub fn follow_system() {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };
    let Ok(Some(media)) = window.match_media(QUERY) else {
        return;
    };
    apply(&document, media.matches());

    // The listener lives as long as the page, so it is leaked rather than stored: there
    // is nothing to unregister it from.
    let listener =
        Closure::<dyn FnMut(web_sys::MediaQueryListEvent)>::new(move |event: web_sys::MediaQueryListEvent| {
            apply(&document, event.matches());
        });
    let _ = media.add_event_listener_with_callback("change", listener.as_ref().unchecked_ref());
    listener.forget();
}

/// Sets the theme daisyUI reads.
fn apply(document: &web_sys::Document, dark: bool) {
    if let Some(root) = document.document_element() {
        let _ = root.set_attribute("data-theme", if dark { "dark" } else { "light" });
    }
}
