//! Infinite scroll: tell a list when its end came into view.
//!
//! One observer per list, created when the sentinel mounts and disconnected with its
//! scope. The callback fires on the entry that is *visible*, and the list itself decides
//! whether it may fetch: a sentinel that stays on screen would otherwise issue one
//! request per observed frame.

use leptos::html;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;

/// Calls `fire` whenever the sentinel becomes visible.
///
/// `fire` is expected to be idempotent while a request is in flight — the guard belongs to
/// the caller, because only the caller knows what "already loading" means.
pub fn when_visible(sentinel: NodeRef<html::Div>, fire: impl Fn() + 'static) {
    let Some(element) = sentinel.get_untracked() else {
        return;
    };

    let callback = Closure::<dyn FnMut(js_sys::Array, wasm_bindgen::JsValue)>::new(
        move |entries: js_sys::Array, _observer: wasm_bindgen::JsValue| {
            let visible = entries.iter().any(|entry| {
                entry
                    .dyn_into::<web_sys::IntersectionObserverEntry>()
                    .is_ok_and(|entry| entry.is_intersecting())
            });
            if visible {
                fire();
            }
        },
    );

    // `root_margin` starts the fetch before the sentinel is on screen, so the next page is
    // usually already there by the time the reader reaches the end.
    let options = web_sys::IntersectionObserverInit::new();
    options.set_root_margin("600px");
    let Some(observer) = web_sys::IntersectionObserver::new_with_options(
        callback.as_ref().unchecked_ref(),
        &options,
    )
    .ok() else {
        return;
    };

    observer.observe(&element);
    // Held for the lifetime of the scope: dropping the closure would leave the observer
    // with a dead callback, and dropping the observer would stop the list from growing.
    on_cleanup(move || {
        observer.disconnect();
    });
    std::mem::forget(callback);
}
