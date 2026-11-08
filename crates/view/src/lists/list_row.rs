//! One list in the picker, shared by the server paint and the client mount.
//!
//! A list row is a name, a replies policy, and the edits the picker offers: select the list,
//! rename it in place, delete it. The last two stay with the screen. A rename *swaps* the
//! title for an input, which a component that renders the title itself cannot express, and
//! the draft and busy signals that drive it are the screen's, so the client passes its form
//! and its delete button through the `children` slot. The server has neither and passes no
//! slot, which draws nothing at all.
//!
//! `selected` is a plain flag rather than a signal: the screen re-renders the row when the
//! selection changes (it is keyed by id in a `For`), so all the row needs is the answer for
//! this render.

use leptos::prelude::*;
use nodus_instance::model::List;

/// One list: its name, its replies policy, and the screen's own controls.
///
/// The title is an anchor with the list's address, so the server's paint is a working link
/// and the client cancels the navigation and selects in place instead. `on_select` receives
/// the list id; the row has no rename or delete of its own, because the verb belongs to the
/// screen that owns the form.
#[component]
pub fn ListRow(
    /// The list this row shows.
    list: List,
    /// Whether the screen is reading this list right now.
    #[prop(optional)]
    selected: bool,
    /// Selects the list inside the app.
    #[prop(optional)]
    on_select: Option<Callback<String>>,
    /// The screen's own inline rename form and delete button.
    #[prop(optional)]
    children: Option<ChildrenFn>,
) -> impl IntoView {
    let List {
        id,
        title,
        replies_policy,
        ..
    } = list;
    let href = format!("/lists/{id}");
    // A stored value is `Copy`, so the same id can be read by the handler and stay usable
    // after `id` has been moved into it.
    let target = StoredValue::new(id);

    view! {
        <li
            class="flex items-center gap-1 rounded-box border border-base-300 bg-base-100 px-2 py-1"
            class:border-primary=selected
        >
            <a
                class="btn btn-ghost btn-xs"
                href=href
                on:click=move |event| select_list(&event, target, on_select)
            >
                {title}
            </a>
            // The server's line: a list's replies policy is not editable from this row, so
            // it is text, not a control.
            <span class="text-xs opacity-60">{replies_policy}</span>
            {children.map(|children| children())}
        </li>
    }
}

/// Selects the list inside the app when there is a callback for it.
///
/// Without one the press is left alone: the `href` is the whole answer, which is what the
/// server's paint relies on (it emits no listener at all). The non-wasm build keeps a use of
/// `event` so the listener argument is not reported as unused on the target that has no
/// `prevent_default`.
fn select_list(
    event: &leptos::ev::MouseEvent,
    id: StoredValue<String>,
    on_select: Option<Callback<String>>,
) {
    #[cfg(not(target_arch = "wasm32"))]
    let _ = event;
    if let Some(on_select) = on_select {
        #[cfg(target_arch = "wasm32")]
        event.prevent_default();
        on_select.run(id.get_value());
    }
}
