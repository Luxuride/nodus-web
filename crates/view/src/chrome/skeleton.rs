//! The placeholder a screen paints while it has nothing to show yet.
//!
//! The server can read a timeline, but not a thread or a notification queue, and the client
//! can arrive at a page it has not loaded. Both paint these grey cards at the same sizes and
//! paddings, so the real content lands in boxes that were already reserved instead of
//! pushing everything below it down when it arrives — which is what "the page jumps" means.
//!
//! There is one card shape, because that is what `render.rs::skeleton` paints for every
//! section: the row count is the only thing a screen chooses.

use leptos::prelude::*;

/// `count` placeholder cards, stacked the way a list of cards is.
#[component]
pub fn Skeleton(count: usize) -> impl IntoView {
    // Nothing to reserve is not an empty list: the screen renders no box at all.
    if count == 0 {
        return ().into_any();
    }
    let cards = std::iter::repeat_with(card)
        .take(count)
        .collect::<Vec<_>>();
    view! { <ul class="flex flex-col gap-2">{cards}</ul> }.into_any()
}

/// One card: the avatar, the two name lines, the body lines and the action boxes of a real
/// card, drawn as filled bars.
fn card() -> AnyView {
    view! {
        <li class="card border border-base-300 bg-base-100 shadow-sm">
            <div class="card-body gap-3 p-4">
                <div class="flex items-center gap-3">
                    <div class="skeleton h-10 w-10 shrink-0 rounded-full"></div>
                    <div class="flex min-w-0 flex-1 flex-col gap-1">
                        <div class="skeleton h-3 w-1/3"></div>
                        <div class="skeleton h-3 w-1/4"></div>
                    </div>
                </div>
                <div class="skeleton h-3 w-full"></div>
                <div class="skeleton h-3 w-4/5"></div>
                <div class="flex gap-2 border-t border-base-200 pt-2">
                    <div class="skeleton h-6 w-16"></div>
                    <div class="skeleton h-6 w-16"></div>
                </div>
            </div>
        </li>
    }
    .into_any()
}
