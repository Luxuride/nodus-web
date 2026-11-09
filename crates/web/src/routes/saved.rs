//! Saved statuses: the two shelves the instance keeps for the signed-in account.
//!
//! The screen is `nodus_view::pages::saved`: the tabs, the shelf they select, the empty state,
//! the list container and the paging control. What stays here is what that crate may not hold
//! — the page store this screen reads, the transport that answers a load, and the card. The
//! card is the client's because each saved status carries state of its own (a verb in flight,
//! its own refusal), so the list is handed a renderer rather than a verb per row.

use std::sync::Arc;

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_sdk::page::{PageItems, PageKind};
use nodus_view::components::CardRenderer;
use nodus_view::pages::saved::{SavedPage, Shelf};

use super::status::StatusCard;
use crate::router::Route;
use crate::state::alive::Alive;
use crate::state::bridge::{pages, revision};

/// The page a shelf is read from. The view crate names the shelf; which request answers it is
/// this crate's business.
fn kind(shelf: Shelf) -> PageKind {
    match shelf {
        Shelf::Favourites => PageKind::Favourites,
        Shelf::Bookmarks => PageKind::Bookmarks,
    }
}

/// Saved statuses, a tab per shelf.
#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "a Leptos component receives its props by value"
)]
pub fn Saved(shelf: String) -> impl IntoView {
    let revision = revision();
    let (shelf, _set_shelf) = signal(Shelf::of(&shelf));
    let (loading, set_loading) =
        signal(!pages::held(&PageKind::Favourites) && !pages::held(&PageKind::Bookmarks));
    let (error, set_error) = signal(None::<String>);

    let held = move || {
        revision.get();
        pages::items(&kind(shelf.get()), |items| match items {
            PageItems::Statuses(list) => Some(list),
            _ => None,
        })
    };
    let statuses = move || held().map(|(list, _)| list).unwrap_or_default();
    let cursor = move || held().and_then(|(_, cursor)| cursor);

    // The busy flag is set from what is held *before* the read starts: a shelf already in the
    // store paints at once, and only an empty one shows the placeholder cards.
    let load = move |which: Shelf, after: Option<String>| {
        let alive = Alive::new();
        set_loading.set(!pages::held(&kind(which)));
        set_error.set(None);
        spawn_local(async move {
            if let Err(message) = pages::ensure_after(kind(which), after).await {
                alive.if_alive(|| set_error.set(Some(message)));
            }
            alive.if_alive(|| set_loading.set(false));
        });
    };
    Effect::new(move |_| load(shelf.get_untracked(), None));

    // Both shelves keep their own cursor, so the press carries the one the reader was shown.
    let on_more = Callback::new(move |after: String| load(shelf.get_untracked(), Some(after)));
    let on_shelf = Callback::new(move |to: Shelf| {
        crate::router::navigate(Route::Saved(to.slug().to_owned()));
    });

    let card: CardRenderer =
        Arc::new(move |status| view! { <StatusCard status=status/> }.into_any());

    view! {
        <SavedPage
            shelf=shelf.into()
            items=Signal::derive(statuses)
            loading=loading.into()
            error=error.into()
            cursor=Signal::derive(cursor)
            card=card
            on_shelf=on_shelf
            on_more=on_more
        />
    }
}
