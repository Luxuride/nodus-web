//! A plain explanatory line, in the one size and shade every screen uses for one.
//!
//! It exists so a screen states "not here yet" or "this build has no moderation screens"
//! with the same box as every other screen, and so the server and the client agree on that
//! box.

use leptos::prelude::*;

#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Leptos components receive their props by value"
)]
pub fn Note(#[prop(optional, into)] class: Option<String>, children: ChildrenFn) -> impl IntoView {
    let class = class.unwrap_or_else(|| "text-sm opacity-70".to_owned());
    view! { <p class=class>{children()}</p> }
}
