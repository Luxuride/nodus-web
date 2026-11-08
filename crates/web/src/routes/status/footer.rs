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
use nodus_view::status_footer::StatusFooter;

use super::menu::StatusMenu;
use super::share::ShareButton;
use crate::router;
use crate::state::alive::Alive;
use crate::state::bridge::actions;

/// Counts on the left, actions on the right.
#[component]
pub fn CardFooter(
    shown: ReadSignal<Status>,
    set_shown: WriteSignal<Status>,
    busy: ReadSignal<bool>,
    set_busy: WriteSignal<bool>,
    set_error: WriteSignal<Option<String>>,
) -> impl IntoView {
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

    // The shared row calls its children again on every repaint of the card, so the slots
    // clone rather than move.
    let menu: ChildrenFn = Arc::new(move || {
        view! { <StatusMenu shown=shown set_error=set_error/> }.into_any()
    });
    let share: ChildrenFn = Arc::new(move || {
        view! { <ShareButton shown=shown set_error=set_error/> }.into_any()
    });

    // The row reads the whole status once, so an answered verb (which replaces the card's
    // copy) repaints the counts and the active highlights.
    view! {
        {move || {
            let status = shown.get();
            let edited = status.edited_at.is_some();
            view! {
                <StatusFooter
                    status=status
                    edited=edited
                    on_reply=on_reply
                    on_boost=on_boost
                    on_favourite=on_favourite
                    on_bookmark=on_bookmark
                    before_verbs=menu.clone()
                    after_verbs=share.clone()
                />
            }
        }}
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
