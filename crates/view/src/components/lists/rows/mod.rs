//! The stack a list of anything is drawn in, and what it says when it is empty.
//!
//! One definition, because this was the last thing both paints wrote by hand: the server
//! concatenated strings into `<ul class="flex flex-col gap-2">`, the client wrote the same
//! class on its own `<ul>`, and the empty state was a `<p class="text-sm opacity-60">` in one
//! place and a `<div>` with different padding in the other. A list that is empty is a state
//! of the same list, so it is rendered here rather than at each call site.
//!
//! `count` rather than a `Signal<Vec<_>>`: a vector of views is not `Send + Sync`, and the
//! emptiness is the only thing this component needs to know about the contents.

use leptos::prelude::*;

/// The class every list of rows is stacked in.
pub const STACK: &str = "flex flex-col gap-2";

/// The class the sentence shown in place of an empty list takes.
pub const EMPTY: &str = "text-sm opacity-60";

/// A list of rows, or the sentence that says there are none.
///
/// The children are already-rendered rows (a `StatusCard`, an `AccountRow`, …), so this stays
/// a layout primitive: it does not know what it is listing.
#[component]
pub fn Rows(count: usize, empty: String, children: ChildrenFn) -> impl IntoView {
    let shows_rows = count > 0;
    view! {
        <Show
            when=move || shows_rows
            fallback=move || view! { <p class=EMPTY>{empty.clone()}</p> }
        >
            <ul class=STACK>{children()}</ul>
        </Show>
    }
}

#[cfg(test)]
mod tests {
    /// The classes are a contract with `web/docs/13-unification-plan.md`: changing one changes
    /// both paints at once, which is the point, so the test pins the strings rather than the
    /// rendered HTML (this crate has no SSR feature of its own).
    #[test]
    fn the_stack_and_the_empty_sentence_keep_their_classes() {
        assert_eq!(super::STACK, "flex flex-col gap-2");
        assert_eq!(super::EMPTY, "text-sm opacity-60");
    }
}
