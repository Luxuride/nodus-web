//! Enabling background sync, which is not a request to us.
//!
//! The service holds a token of its own (D37), so "on" means the user granted *that*
//! application access on their instance. The browser can only open the consent screen:
//! the answer arrives at the service's callback, and the card re-reads `/me` when the
//! user comes back.

use leptos::prelude::*;

use crate::state::alive::Alive;
use crate::state::bridge::authorize_sync;

/// Opens the instance's authorization screen, or reports why it could not.
///
/// `result` and `busy` are the card's own signals; `alive` is checked before every
/// write because the user may navigate away while the instance is answering.
pub(super) fn start(result: RwSignal<Option<String>>, busy: RwSignal<bool>, alive: Alive) {
    wasm_bindgen_futures::spawn_local(async move {
        match authorize_sync().await {
            Ok(url) => {
                if let Some(window) = web_sys::window() {
                    // A new tab: the instance's consent screen is not part of this app,
                    // and leaving the current page would lose the card that explains it.
                    let _ = window.open_with_url_and_target(&url, "_blank");
                }
                alive.if_alive(|| {
                    busy.set(false);
                    result.set(Some(
                        "Approve Nodus on your instance, then come back — this card updates when you do."
                            .to_owned(),
                    ));
                });
            }
            Err(message) => alive.if_alive(|| {
                busy.set(false);
                result.set(Some(format!(
                    "Could not start the authorization: {message}"
                )));
            }),
        }
    });
}
