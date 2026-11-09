//! The foot of a paged screen: the way to the page after this one.

use leptos::prelude::*;

/// The control that asks for the next page, or nothing when this is the last one.
///
/// `on_more` is the whole difference between the two paints. The client has it, so the foot
/// is a button that says what it is doing while the next page is on its way; the server does
/// not, so the same foot is an anchor whose `href` carries the cursor and the page still
/// paginates without a line of JavaScript. The class string is one either way, so a reader
/// who lands on a painted page and a reader who has been paging for an hour see the control
/// in the same place.
#[component]
pub fn MoreButton(
    /// The cursor of the next page, or `None` when this is the last one.
    #[prop(optional)]
    cursor: Option<String>,
    /// Whether the next page is already on its way.
    loading: bool,
    /// Asks for the page after `cursor`. Absent on the server.
    #[prop(optional)]
    on_more: Option<Callback<String>>,
) -> impl IntoView {
    // No cursor, no next page: the foot is absent, never a control that does nothing.
    let Some(cursor) = cursor else {
        return ().into_any();
    };
    let label = if loading { "Loading…" } else { "Load more" };
    if let Some(on_more) = on_more {
        // The cursor is cloned per press because a callback is `Fn` and the press can come
        // more than once.
        return view! {
            <button
                class="btn btn-ghost btn-sm w-full"
                disabled=loading
                on:click=move |_| on_more.run(cursor.clone())
            >
                {label}
            </button>
        }
        .into_any();
    }
    view! {
        <a class="btn btn-ghost btn-sm w-full" href=format!("?cursor={cursor}")>
            {label}
        </a>
    }
    .into_any()
}
