//! One invisible row that asks for the next page when it reaches the viewport.

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

    #[cfg(target_arch = "wasm32")]
    {
        Effect::new(move |_| {
            let Some(element) = node.get() else {
                return;
            };
            let callback = wasm_bindgen::closure::Closure::<
                dyn FnMut(js_sys::Array, web_sys::IntersectionObserver),
            >::new(
                move |entries: js_sys::Array, _observer: web_sys::IntersectionObserver| {
                    if !active.get_untracked() {
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
            on_cleanup(move || observer.disconnect());
        });
    }

    view! { <div node_ref=node class="h-px w-full"></div> }
}
