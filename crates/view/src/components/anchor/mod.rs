//! A relatively positioned box whose element the caller needs to keep.
//!
//! The document listeners that close a popover ask whether a click landed inside it, and
//! that question needs the element itself. The box is markup, so it lives here; the
//! listener and its `Closure` stay with the caller.

use leptos::prelude::*;

/// A relative box that the caller can test a click against.
#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Leptos components receive their props by value"
)]
pub fn Anchor(host: NodeRef<leptos::html::Div>, children: ChildrenFn) -> impl IntoView {
    view! {
        <div class="relative" node_ref=host>
            {children()}
        </div>
    }
}

/// A relative box, for a panel positioned against the box rather than against the page.
#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Leptos components receive their props by value"
)]
pub fn Relative(children: ChildrenFn) -> impl IntoView {
    view! {
        <div class="relative">{children()}</div>
    }
}
