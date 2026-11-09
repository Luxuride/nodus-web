//! The saved screen: a tab per shelf and one list beneath it.
//!
//! Favourites and bookmarks are separate endpoints with the same shape, so they share a
//! screen with a tab each. The address decides the shelf and the caller passes what is held
//! for it, because the page may not read a page store: switching shelves is a navigation, and
//! the page reports the press rather than moving the reader itself.
//!
//! The list is the caller's to paint. A saved status carries verbs, a media viewer and a clock
//! with it, and the server has none of those — so the card arrives as a [`CardRenderer`] and
//! the page keeps only what both paints agree on: the container, the keys, the empty state,
//! the paging control and the error line.

#[path = "../shelf/mod.rs"]
mod shelf;

pub use shelf::Shelf;

use leptos::prelude::*;
use nodus_instance::model::Status;

use crate::chrome::Skeleton;
use crate::chrome::more_button::{MoreButton, MoreButtonProps};
use crate::components::CardRenderer;
use crate::{PageHeader, Screen};

/// The saved screen, with its two shelves.
#[component]
pub fn SavedPage(
    /// The shelf the address names.
    shelf: Signal<Shelf>,
    /// The statuses held for that shelf, newest first.
    items: Signal<Vec<Status>>,
    /// Whether a read is already on its way.
    loading: Signal<bool>,
    /// The last read's refusal, when it had one.
    error: Signal<Option<String>>,
    /// The cursor of the page after this one, when the instance has more.
    cursor: Signal<Option<String>>,
    /// Paints one status. The page cannot: a card with verbs, a media viewer and a clock is
    /// the client's, and a server hands in the same card with nothing behind it.
    card: CardRenderer,
    /// Navigates to another shelf.
    #[prop(optional)]
    on_shelf: Option<Callback<Shelf>>,
    /// Asks for the page after the given cursor. Absent on the server, where the control
    /// keeps its box and becomes a link.
    #[prop(optional)]
    on_more: Option<Callback<String>>,
) -> impl IntoView {
    // An empty shelf says which shelf it is: the two tabs are one screen, and "nothing yet"
    // without a subject would be about the wrong one.
    let empty = move || shelf.get().empty();

    // The card is one value the whole list shares, kept where a repeating row can call it
    // without taking it: a list repaints, and the renderer is only ever borrowed.
    let card = StoredValue::new(card);

    view! {
        <Screen>
            <PageHeader title="Saved" description="Your favourites and bookmarks, newest first."/>

            <div role="tablist" class="tabs tabs-box w-fit">
                <For each=|| Shelf::ALL key=|which| which.slug() let:which>
                    <a
                        role="tab"
                        class="tab"
                        href=format!("/saved/{}", which.slug())
                        class:tab-active=move || shelf.get() == which
                        on:click=move |event| pick_shelf(&event, which, on_shelf)
                    >
                        {which.label()}
                    </a>
                </For>
            </div>

            // The placeholder cards are the first paint only: a shelf the caller already
            // holds renders at once, and a shelf that came back empty is a fact, not a wait.
            <Show when=move || loading.get() && items.get().is_empty()>
                <Skeleton count=4/>
            </Show>

            <Show when=move || !loading.get() && items.get().is_empty() && error.get().is_none()>
                <p class="text-sm opacity-60">{empty}</p>
            </Show>

            <ul class="flex flex-col gap-3">
                <For each=move || items.get() key=|status| status.id.clone() let:status>
                    {card.with_value(|card| card(status))}
                </For>
            </ul>

            {move || {
                let next = cursor.get();
                // The paging control is handed its whole set of props: view syntax can pass an
                // affordance but not the absence of one, and this page only knows what it was
                // given.
                MoreButton(MoreButtonProps {
                    cursor: next,
                    loading: loading.get(),
                    on_more,
                })
            }}

            <Show when=move || error.get().is_some()>
                <p class="text-error text-xs">{move || error.get().unwrap_or_default()}</p>
            </Show>
        </Screen>
    }
}

/// Opens another shelf, leaving a modified press to the browser.
///
/// Each tab is a real address, so the server's paint is already a working link. On the client
/// the press is a step inside the app instead; a press with a modifier is the reader asking
/// for a new tab or window, and the app would take that away by cancelling it.
fn pick_shelf(event: &leptos::ev::MouseEvent, which: Shelf, on_shelf: Option<Callback<Shelf>>) {
    // The non-wasm build keeps a use of `event` so the listener argument is not reported as
    // unused on the target that has no `prevent_default`.
    #[cfg(not(target_arch = "wasm32"))]
    let _ = event;
    let Some(on_shelf) = on_shelf else {
        return;
    };
    #[cfg(target_arch = "wasm32")]
    if event.button() != 0
        || event.meta_key()
        || event.ctrl_key()
        || event.shift_key()
        || event.alt_key()
    {
        return;
    }
    #[cfg(target_arch = "wasm32")]
    event.prevent_default();
    on_shelf.run(which);
}
