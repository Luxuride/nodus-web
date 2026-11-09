//! What a confirmed intent actually does.
//!
//! Enabling leaves this app entirely (the user authorizes the service on their
//! instance, [`super::authorize`]); disabling is one request that deletes the copy.
//! Kept apart from the card so the card is markup and this is the decision.

use leptos::prelude::*;

use nodus_view::pages::settings::sync::SyncIntent;

use super::authorize;
use crate::state::alive::Alive;
use crate::state::bridge::set_sync;

/// Runs `intent` once the user confirmed it.
pub(super) fn run(
    intent: SyncIntent,
    confirming: RwSignal<Option<SyncIntent>>,
    result: RwSignal<Option<String>>,
    busy: RwSignal<bool>,
    alive: Alive,
) {
    if busy.get_untracked() {
        return;
    }
    confirming.set(None);
    busy.set(true);
    result.set(None);

    if intent.enables() {
        authorize::start(result, busy, alive);
        return;
    }

    wasm_bindgen_futures::spawn_local(async move {
        let message = match set_sync(false, false).await {
            Ok(()) => {
                "Background sync is off, and the server deleted its copy of your token.".to_owned()
            }
            Err(error) => format!("Could not turn background sync off: {error}"),
        };
        alive.if_alive(|| {
            result.set(Some(message));
            busy.set(false);
        });
    });
}
