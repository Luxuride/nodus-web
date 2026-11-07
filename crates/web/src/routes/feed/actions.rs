//! The four things a feed card can do, kept out of the markup.
//!
//! Each handler owns a small async round trip and reports through one of three
//! channels: `error` for something the card shows in place, `toast` for the
//! transient copy-link report, and the follow flip for the parent's remount. They
//! live here rather than in `item.rs` so the card's markup stays readable as markup.

use std::time::Duration;

use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::MouseEvent;

use crate::state::alive::Alive;
use crate::state::bridge::{follow_item, not_interested, unfollow_item, with_sdk};

use super::share::{ShareReport, TOAST_MILLIS, copy_profile_link};

/// The click handlers a card wires into its buttons.
pub(super) struct ItemActions {
    pub follow: Box<dyn FnMut(MouseEvent) + 'static>,
    pub unfollow: Box<dyn FnMut(MouseEvent) + 'static>,
    pub not_interested: Box<dyn FnMut(MouseEvent) + 'static>,
    pub copy: Box<dyn FnMut(MouseEvent) + 'static>,
}

/// Builds the handlers for one item.
///
/// One `busy` flag guards every handler: the three network actions are mutually
/// exclusive in practice (a user who has just tapped Follow should not be able to
/// fire Not interested at the same item a millisecond later), and the copy action
/// is local, so it does not disable the others.
#[expect(
    clippy::needless_pass_by_value,
    reason = "the identifier is cloned into three closures and one async block"
)]
pub(super) fn item_actions(
    item_id: i64,
    account_id: String,
    acct: String,
    busy: RwSignal<bool>,
    error: RwSignal<Option<String>>,
    toast: RwSignal<Option<ShareReport>>,
    alive: Alive,
) -> ItemActions {
    let follow = {
        let account_id = account_id.clone();
        Box::new(move |_| {
            if busy.get_untracked() {
                return;
            }
            busy.set(true);
            error.set(None);
            let account_id = account_id.clone();

            spawn_local(async move {
                let result = follow_item(item_id, &account_id).await;
                alive.if_alive(|| busy.set(false));
                if let Err(message) = result {
                    alive.if_alive(|| error.set(Some(message)));
                }
            });
        }) as Box<dyn FnMut(MouseEvent)>
    };

    let unfollow = {
        let account_id = account_id.clone();
        Box::new(move |_| {
            if busy.get_untracked() {
                return;
            }
            busy.set(true);
            let account_id = account_id.clone();

            spawn_local(async move {
                // Failure surfaces in the Feed tab's alert, not here: the follow
                // flip remounts this card (see `Feed`'s list key), so a message
                // held in `error` would be dropped before it could be read.
                unfollow_item(account_id).await;
                alive.if_alive(|| busy.set(false));
            });
        }) as Box<dyn FnMut(MouseEvent)>
    };

    let not_interested = Box::new(move |_| {
        if busy.get_untracked() {
            return;
        }
        busy.set(true);
        error.set(None);

        spawn_local(async move {
            let result = not_interested(item_id).await;
            alive.if_alive(|| busy.set(false));
            if let Err(message) = result {
                alive.if_alive(|| error.set(Some(message)));
            }
        });
    }) as Box<dyn FnMut(MouseEvent)>;

    let copy = Box::new(move |_| {
        toast.set(None);
        let acct = acct.clone();
        // Read at click time, not at render time: the session may have finished
        // restoring after this card was built.
        let host = with_sdk(|sdk| sdk.state().session.get().map(|session| session.host));

        spawn_local(async move {
            let Some(host) = host else {
                alive.if_alive(|| {
                    toast.set(Some(ShareReport {
                        ok: false,
                        // The same sentence `share` uses: a card cannot be copied
                        // without the instance the profile lives on.
                        message: "Not signed in.".to_owned(),
                    }));
                });
                set_timeout(move || toast.set(None), Duration::from_millis(TOAST_MILLIS));
                return;
            };
            let report = match copy_profile_link(&host, &acct).await {
                Ok(()) => ShareReport {
                    ok: true,
                    message: "Profile link copied".to_owned(),
                },
                Err(message) => ShareReport { ok: false, message },
            };
            alive.if_alive(|| toast.set(Some(report)));
            set_timeout(move || toast.set(None), Duration::from_millis(TOAST_MILLIS));
        });
    }) as Box<dyn FnMut(MouseEvent)>;

    ItemActions {
        follow,
        unfollow,
        not_interested,
        copy,
    }
}
