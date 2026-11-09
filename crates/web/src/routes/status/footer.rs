//! The action row under a status.
//!
//! Each button runs one verb and hands the answer back to the card. There is no
//! optimistic update: a favourite that the server refused would leave the button lit
//! with nothing to undo it (the same reason the SDK dispatches consent only after a
//! successful request).
//!
//! The row's markup lives in `nodus-view` because the server paints it too, so the two
//! paints cannot drift apart on the counts or the button boxes. Everything the shared
//! component must not know about stays here: the card's signals, the transport, and the
//! two controls that exist only in the app (the menu and the copy button), which arrive
//! as the row's slots.

use std::future::Future;
use std::sync::Arc;

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::Status;

use super::menu::StatusMenu;
use super::share::ShareButton;
use crate::router;
use crate::state::alive::Alive;
use crate::state::bridge::actions;

/// The handlers a card's action row runs, ready to hand to the shared card.
pub(super) struct CardActions {
    /// Opens the thread.
    pub on_reply: Callback<String>,
    /// Boosts, or undoes the boost.
    pub on_boost: Callback<String>,
    /// Favourites, or removes the favourite.
    pub on_favourite: Callback<String>,
    /// Bookmarks, or removes the bookmark.
    pub on_bookmark: Callback<String>,
    /// The post's menu, drawn where it sits in the row.
    pub before_verbs: ChildrenFn,
    /// The copy button, drawn after the verbs.
    pub after_verbs: ChildrenFn,
}

/// Builds the row's handlers from the card's signals.
///
/// The shared card draws the row, so this returns what the row needs rather than a view: the
/// three verbs run one call each and fold the answer back into the card, and the two controls
/// that exist only in the app — the menu and the copy button — arrive as the row's slots,
/// because the shared component may not reach for the transport or the clipboard.
pub(super) fn card_actions(
    shown: ReadSignal<Status>,
    set_shown: WriteSignal<Status>,
    busy: ReadSignal<bool>,
    set_busy: WriteSignal<bool>,
    set_error: WriteSignal<Option<String>>,
) -> CardActions {
    let alive = Alive::new();

    let on_reply = Callback::new(move |id: String| router::navigate(super::thread_route(&id)));

    let on_boost = Callback::new(move |id: String| {
        if busy.get_untracked() {
            return;
        }
        let want = !shown.get_untracked().reblogged;
        dispatch(
            async move { actions::reblog(&id, want).await },
            alive,
            set_shown,
            set_busy,
            set_error,
        );
    });

    let on_favourite = Callback::new(move |id: String| {
        if busy.get_untracked() {
            return;
        }
        let want = !shown.get_untracked().favourited;
        dispatch(
            async move { actions::favourite(&id, want).await },
            alive,
            set_shown,
            set_busy,
            set_error,
        );
    });

    let on_bookmark = Callback::new(move |id: String| {
        if busy.get_untracked() {
            return;
        }
        let want = !shown.get_untracked().bookmarked;
        dispatch(
            async move { actions::bookmark(&id, want).await },
            alive,
            set_shown,
            set_busy,
            set_error,
        );
    });

    // A `ChildrenFn` is cloned on every repaint of the card, so the slots clone rather than
    // move. Both are present here: the app always has a menu and a copy button, and the
    // absent case belongs to the paint that has no transport.
    let menu: ChildrenFn =
        Arc::new(move || view! { <StatusMenu shown=shown set_error=set_error/> }.into_any());
    let share: ChildrenFn =
        Arc::new(move || view! { <ShareButton shown=shown set_error=set_error/> }.into_any());

    CardActions {
        on_reply,
        on_boost,
        on_favourite,
        on_bookmark,
        before_verbs: menu,
        after_verbs: share,
    }
}

/// Runs one verb and folds its answer back into the card.
///
/// The three verbs differ only in the call they make, so marking the card busy, folding
/// the answer in and clearing the flag live once. The writes go through [`Alive`] because
/// the answer arrives after the reader may have left the screen.
fn dispatch(
    call: impl Future<Output = Result<Status, String>> + 'static,
    alive: Alive,
    set_shown: WriteSignal<Status>,
    set_busy: WriteSignal<bool>,
    set_error: WriteSignal<Option<String>>,
) {
    set_busy.set(true);
    set_error.set(None);

    spawn_local(async move {
        match call.await {
            Ok(status) => alive.if_alive(|| set_shown.set(status)),
            Err(message) => alive.if_alive(|| set_error.set(Some(message))),
        }
        alive.if_alive(|| set_busy.set(false));
    });
}
