//! A settings card: a titled box that holds controls.
//!
//! Every settings screen is built from this shape, and the shapes had drifted apart one
//! class at a time (`gap-2` here, `gap-3` there, a `mb-4` that only one card carried). One
//! component means a new card looks like the others by construction, and the difference
//! between two cards is what they contain rather than which padding they chose.

use leptos::prelude::*;

/// A titled card. `description` is the one-line explanation under the title.
#[component]
pub fn Panel(
    #[prop(into)] title: String,
    #[prop(optional, into)] description: Option<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <section class="card border border-base-300 bg-base-100">
            <div class="card-body gap-3 p-4">
                <h2 class="card-title text-base">{title}</h2>
                {description.map(|text| view! { <p class="text-sm opacity-70">{text}</p> })}
                {children()}
            </div>
        </section>
    }
}
