//! The foot of a list that pages in place: one press, no address to carry.
//!
//! [`super::more_button::MoreButton`] puts the cursor in its `href` so that a server can
//! paginate a list it never read. A profile's posts are not a document with a cursor in it —
//! the screen owns the list and its cursor — so this foot is always a button, and a press
//! asks that screen for the page after the one it holds.

use leptos::prelude::*;

/// "Load more", disabled while the next page is on its way.
///
/// The button is drawn even without `on_more`, so the two paints differ by an affordance and
/// not by a box.
#[component]
pub fn LoadMore(
    /// Whether the next page is already on its way.
    loading: Signal<bool>,
    /// Asks the screen for its next page. Absent where there is nowhere to ask.
    #[prop(optional)]
    on_more: Option<Callback<()>>,
) -> impl IntoView {
    view! {
        <button
            class="btn btn-ghost btn-sm w-full"
            disabled=move || loading.get() || on_more.is_none()
            on:click=move |_| {
                if let Some(on_more) = on_more {
                    on_more.run(());
                }
            }
        >
            "Load more"
        </button>
    }
}
