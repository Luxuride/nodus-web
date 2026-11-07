//! The browser-local session record and the verbs that change the session.
//!
//! The SDK keeps the live session; this module keeps the *remembered* one, so a
//! reload does not make the user paste a token again. `record` owns the storage
//! format; this file owns the verbs, and `state::bridge` re-exports them so call
//! sites keep saying `bridge::sign_in` and friends.

use leptos::prelude::*;
use nodus_instance::BrowserInstance;
use nodus_sdk::instance_api::{Instance, InstanceApi};

use self::record::StoredSession;
use super::feed::load_first_page;
use super::{WebSdkError, feed_ui, handle, install};

mod record;

/// Re-establishes a stored session without blocking the first paint.
///
/// Called by [`super::init`] once the unauthenticated SDK exists. A host we can no
/// longer parse (or a hand-edited record) is not a session: it is dropped rather
/// than failing every boot.
pub(super) fn restore_saved() {
    let Some(stored) = record::load() else {
        return;
    };
    let Ok(instance) = Instance::parse(&stored.host) else {
        clear_session();
        return;
    };
    if install(&instance, &stored.token).is_err() {
        return;
    }
    restore(stored);
}

/// Signs in with a pasted token and loads the first page of the feed.
///
/// The token is verified against the instance on a throwaway transport *before*
/// anything is persisted or installed, so a rejected token cannot replace the SDK
/// the rest of the app is rendering from. Only a token the instance accepted is
/// stored (see [`save_session`]) and only then does the SDK see a session.
///
/// # Errors
///
/// Returns a message fit for an inline error: bad input, an unparseable host, a
/// rejected token, or a failed bridge install. Nothing is saved on error.
pub async fn sign_in(host: &str, token: &str) -> Result<(), String> {
    let host = host.trim();
    let token = token.trim();
    if host.is_empty() || token.is_empty() {
        return Err("An instance host and an access token are both required.".to_owned());
    }

    let instance = Instance::parse(host).map_err(|error| error.to_string())?;
    let probe = BrowserInstance::new(&instance, token);
    probe
        .verify_credentials()
        .await
        .map_err(|error| format!("The instance rejected this token: {error}"))?;

    install(&instance, token)?;

    let Some(sdk) = handle() else {
        return Err("The SDK is not running.".to_owned());
    };
    let ui = feed_ui();
    ui.error.set(None);
    sdk.sign_in(host, scopes())
        .await
        .map_err(|error| error.to_string())?;
    // After `sign_in`, because the account handle it returns is what a restored
    // session needs to display before its first request.
    save_session(host, token);

    ui.refreshing.set(true);
    load_first_page(&sdk, ui).await;
    Ok(())
}

/// Signs out: forgets the stored session and tells the SDK.
pub fn sign_out() {
    clear_session();
    let Some(sdk) = handle() else {
        return;
    };
    wasm_bindgen_futures::spawn_local(async move {
        // `sign_out` dispatches `SignedOut` before awaiting the cache clear, so the
        // UI flips on the next microtask; a cache failure has nothing left to show.
        let _ = sdk.sign_out().await;
    });
}

/// Turns server-side sync on or off, with telemetry as a separate consent.
///
/// # Errors
///
/// Returns a message fit for inline display (typically not signed in, or the API
/// refused the request).
pub async fn set_sync(enable: bool, telemetry_opt_in: bool) -> Result<(), String> {
    let sdk = handle().ok_or_else(|| "Not signed in.".to_owned())?;
    sdk.set_sync(enable, telemetry_opt_in)
        .await
        .map_err(|error| error.to_string())
}

/// Asks the server what it holds for this account (sync consent, telemetry).
///
/// Settings calls this on open so the toggle describes the server: another browser
/// may have turned sync off, and a toggle that only remembers this tab would lie.
pub async fn load_sync_consent() -> Result<(), String> {
    let sdk = handle().ok_or_else(|| "Not signed in.".to_owned())?;
    sdk.load_sync_consent()
        .await
        .map(drop)
        .map_err(|error| error.to_string())
}

/// Remembers the session for the next page load.
///
/// # Milestone-1 caveat
///
/// The token lives in `localStorage` in plain text. That is a deliberate
/// milestone-1 trade-off so a reload does not make the user paste a token again;
/// the PKCE + `IndexedDB` design in `web/docs/03-data-layer.md` replaces it —
/// there the browser holds a revocable OAuth token and the secret never reaches a
/// JS-readable string. Until then this is the only place that touches storage, and
/// the sign-in view goes through it rather than calling `localStorage` itself.
///
/// A storage failure (full, disabled, private mode) is not fatal: the session
/// works for this page load and the next one asks for the token again.
pub fn save_session(host: &str, token: &str) {
    record::store(host, token);
}

/// Forgets the stored session. Does not touch the SDK; see [`sign_out`] for that.
pub fn clear_session() {
    record::clear();
    super::forget_feed_id();
    // An impression names a user who is signing out; the count would be against
    // nobody, and the next session must not inherit the previous list's queue.
    super::impressions::forget();
}

/// Re-establishes a stored session without blocking the first paint.
///
/// The SDK's `Session` carries the account id, which a local record would be
/// lying about if the account changed, so the restore asks the instance who the
/// token belongs to (that is what `Sdk::sign_in` does) and then loads the feed.
fn restore(stored: StoredSession) {
    let Some(sdk) = handle() else {
        return;
    };
    let ui = feed_ui();
    ui.refreshing.set(true);
    wasm_bindgen_futures::spawn_local(async move {
        match sdk.sign_in(&stored.host, scopes()).await {
            Ok(_) => load_first_page(&sdk, ui).await,
            Err(error) => {
                // A token the instance rejected is dead: forget it so the next boot
                // does not replay it. Anything else (offline, server down) keeps it,
                // because the user's consent has not changed.
                if matches!(
                    error,
                    WebSdkError::Instance(nodus_sdk::instance_api::Error::Unauthorized)
                ) {
                    clear_session();
                }
                ui.error
                    .set(Some(format!("Saved session did not restore: {error}")));
                ui.refreshing.set(false);
            }
        }
    });
}

/// The scopes a pasted token is assumed to carry.
///
/// The instance's token page hands out `read write follow`; OAuth will let the
/// user narrow this, and PKCE will request exactly what the app uses.
fn scopes() -> Vec<String> {
    vec!["read".to_owned(), "write".to_owned(), "follow".to_owned()]
}
