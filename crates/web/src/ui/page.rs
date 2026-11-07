//! The heading a screen opens with.

use leptos::prelude::*;

/// A title and one sentence of orientation.
///
/// The sentence is not decoration: a reader arriving from a link needs to know what the
/// screen is for before it finishes loading, and every empty state on the screen is
/// cheaper to read once that is on the page.
#[component]
pub fn PageHeader(title: &'static str, description: &'static str) -> impl IntoView {
    view! {
        <header class="flex flex-col gap-0.5">
            <h1 class="text-lg font-semibold">{title}</h1>
            <p class="text-sm opacity-60">{description}</p>
        </header>
    }
}
