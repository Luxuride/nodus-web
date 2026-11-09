//! The bordered box a composer's writing area sits in.

use leptos::prelude::*;

/// A field's frame: the same border and padding wherever a writer types.
#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Leptos components receive their props by value"
)]
pub fn FieldBox(children: ChildrenFn) -> impl IntoView {
    view! {
        <div class="rounded-box border border-base-300 bg-base-100 p-3">{children()}</div>
    }
}
