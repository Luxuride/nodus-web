//! A panel that hangs under a control and is mounted by the caller's script.
//!
//! The emoji picker is drawn by a script into an element the client keeps a handle to; the
//! frame around that element is markup, so it lives here. Whether the panel is open is the
//! caller's signal, because the same signal also drives the document listener.

use leptos::prelude::*;

/// The framed area a script mounts its picker into.
#[component]
pub fn Popover(
    #[prop(into)] open: Signal<bool>,
    host: NodeRef<leptos::html::Div>,
    #[prop(optional, into)] class: Option<String>,
) -> impl IntoView {
    let class = class.unwrap_or_else(|| {
        "absolute right-0 top-full z-50 mt-1 w-80 max-w-[calc(100vw-1rem)] rounded-box border border-base-300 bg-base-100 p-2 shadow-xl".to_owned()
    });
    view! {
        <Show when=move || open.get()>
            <div class=class.clone()>
                <div node_ref=host></div>
            </div>
        </Show>
    }
}
