//! The browser-local session record and the verbs that change the session.
//!
//! The SDK keeps the live session; this module keeps the *remembered* one, so a
//! reload does not make the user paste a token again. `record` owns the storage
//! format; this file owns the verbs, and `state::bridge` re-exports them so call
//! sites keep saying `bridge::sign_in` and friends.

use leptos::prelude::*;
use nodus_instance::BrowserInstance;
use nodus_sdk::instance_api::{Instance, InstanceApi};
use nodus_sdk::state::Session;

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
    adopt(&stored);
    restore(stored);
}

/// Shows the remembered session before the instance has confirmed it.
///
/// Without this the app renders the sign-in screen from first paint until
/// `verify_credentials` answers, which is the flash the user sees on every reload.
/// The record is the browser's own memory of a session the instance granted; the
/// token is what the instance actually checks, so adopting it can only ever be
/// cosmetic between one request and the next.
fn adopt(stored: &StoredSession) {
    // The host is what makes a bare `/status/{id}` readable next time, so write it before
    // anything can fail.
    record::remember_host(&stored.host);
    if stored.account_id.is_empty() {
        return;
    }
    let Some(sdk) = handle() else {
        return;
    };
    sdk.adopt_session(Session {
        host: stored.host.clone(),
        account_id: stored.account_id.clone(),
        acct: stored.acct.clone(),
        scopes: scopes(),
    });
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
    let session = sdk
        .sign_in(host, scopes())
        .await
        .map_err(|error| error.to_string())?;
    // After `sign_in`, because the account handle it returns is what a restored
    // session needs to display before its first request.
    save_session(host, token, &session);

    ui.refreshing.set(true);
    load_first_page(&sdk, ui).await;
    Ok(())
}

/// Signs out: forgets the stored session and tells the SDK.
pub fn sign_out() {
    let Some(sdk) = handle() else {
        clear_session();
        return;
    };
    // The account being signed out is the one on screen, and it is the only one this
    // gesture may forget: the others keep their tokens and their SDKs.
    if let Some(key) = super::active_key() {
        super::forget(&key);
    }
    clear_session();
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
pub fn save_session(host: &str, token: &str, session: &Session) {
    record::store(host, token, &session.account_id, &session.acct);
    record::remember_host(host);
    // Keep the account in the switcher too, so the next visit can offer it without a
    // second sign-in. The SDK that was just installed is now named.
    super::register(super::accounts::Account {
        host: host.to_owned(),
        token: token.to_owned(),
        id: session.account_id.clone(),
        acct: session.acct.clone(),
    });
}

/// The instance this browser last read from.
///
/// Used by the anonymous post view: a shared `/status/{id}` link has no host, and the
/// only defensible guess is the instance the reader already used.
pub fn remembered_host() -> Option<String> {
    // A live session knows its host; the record is a fallback for a reader who has never
    // signed in here (or has since signed out).
    if let Some(host) =
        handle().and_then(|sdk| sdk.state().session.get().map(|session| session.host))
    {
        return Some(host);
    }
    record::remembered_host()
}

/// Starts a *separate* authorization for background sync (D37).
///
/// The service registers its own application and asks the user to approve it, so the
/// token it stores is scoped to what it does (`read`) instead of being a copy of this
/// browser's token. Returns the instance URL to send the user to.
///
/// # Errors
///
/// A message fit for inline display: not signed in, the request failed, or the service
/// answered something that is not a URL.
pub async fn authorize_sync() -> Result<String, String> {
    let Some(stored) = record::load() else {
        return Err("Sign in on this browser first.".to_owned());
    };
    let base = super::api_base().ok_or_else(|| "No API base URL.".to_owned())?;
    let response = gloo_net::http::Request::post(
        base.join("api/v1/sync/authorize")
            .map_err(|error| error.to_string())?
            .as_str(),
    )
    .header("authorization", &format!("Bearer {}", stored.token))
    .header("x-instance-host", &stored.host)
    .header("x-contract-version", nodus_shared::CONTRACT_VERSION)
    .send()
    .await
    .map_err(|error| format!("Could not reach the service: {error}"))?;
    if !response.ok() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("The service answered {status}: {body}"));
    }
    // The browser's own parser, as everywhere else in this crate: one field is not
    // worth a deserializer in the bundle.
    let body = response
        .text()
        .await
        .map_err(|error| format!("The answer could not be read: {error}"))?;
    let parsed = js_sys::JSON::parse(&body)
        .map_err(|_| "The service did not answer with JSON.".to_owned())?;
    js_sys::Reflect::get(&parsed, &wasm_bindgen::JsValue::from_str("url"))
        .ok()
        .and_then(|url| url.as_string())
        .ok_or_else(|| "The service did not answer with an authorize URL.".to_owned())
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
            Ok(session) => {
                // The instance is the authority: if the account behind the token
                // changed, the record (and the header drawn from it) follows.
                if session.account_id != stored.account_id || session.acct != stored.acct {
                    save_session(&stored.host, &stored.token, &session);
                }
                load_first_page(&sdk, ui).await;
            }
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
