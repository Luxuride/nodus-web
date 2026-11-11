//! One invisible row that asks for the next page when it reaches the viewport.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use leptos::html::Div;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

#[component]
pub fn Infinite(
    /// Whether there is another page to ask for right now.
    active: Signal<bool>,
    on_load: Callback<()>,
) -> impl IntoView {
    let node = NodeRef::<Div>::new();
    let live = Arc::new(AtomicBool::new(true));

    #[cfg(target_arch = "wasm32")]
    {
        Effect::new(move |_| {
            let alive = Arc::clone(&live);
            let Some(element) = node.get() else {
                return;
            };
            let callback = wasm_bindgen::closure::Closure::<
                dyn FnMut(js_sys::Array, web_sys::IntersectionObserver),
            >::new(
                move |entries: js_sys::Array, _observer: web_sys::IntersectionObserver| {
                    if !alive.load(Ordering::Relaxed) || !active.get_untracked() {
                        return;
                    }
                    let visible = entries.iter().any(|entry| {
                        entry
                            .dyn_into::<web_sys::IntersectionObserverEntry>()
                            .is_ok_and(|entry| entry.is_intersecting())
                    });
                    if visible {
                        on_load.run(());
                    }
                },
            );
            let Ok(observer) = web_sys::IntersectionObserver::new(
                callback.as_ref().unchecked_ref::<js_sys::Function>(),
            ) else {
                return;
            };
            observer.observe(&element);
            callback.forget();
            let cleanup_live = Arc::clone(&live);
            on_cleanup(move || {
                cleanup_live.store(false, Ordering::Relaxed);
                observer.disconnect();
            });
        });
    }

    view! { <div node_ref=node class="h-px w-full"></div> }
}
