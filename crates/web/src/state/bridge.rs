//! Glue between the SDK (source of truth) and Leptos (renderer).
//!
//! The SDK is deliberately framework-agnostic: its state is `Rc<RefCell<_>>`
//! handles, which are single-threaded and therefore not usable directly as Leptos
//! signals (Leptos 0.8 signals must be `Send + Sync`). This module is the one
//! place that bridges the two, and it does so with a single revision counter:
//!
//! * the SDK owns the data and notifies subscribers on change;
//! * each subscription bumps a `RwSignal<u64>` revision;
//! * a view renders by reading the revision (which subscribes the view to it) and
//!   then reading the data straight out of the SDK.
//!
//! Views therefore never copy state and never keep their own cache — they read SDK
//! state and dispatch commands, which is exactly the split we want.
//!
//! This module also owns the two things the SDK deliberately does not: the
//! browser-local session record (milestone 1, see [`save_session`]) and the
//! transient request flags of the Feed tab ([`feed_ui`]) — "refreshing" and "the
//! last request failed" describe a request, not the data.
//!
//! See `web/docs/02-architecture.md` ("SDK owns state") and `sdk/README.md`.

use std::cell::RefCell;
use std::rc::Rc;

use leptos::prelude::*;
use nodus_discovery::BrowserDiscovery;
use nodus_instance_rest::BrowserInstance;
use nodus_sdk::instance_api::{Instance, LocalId};
use nodus_sdk::{Command, InstanceApi, MemoryPersistence, Sdk, Subscription, SystemClock};
use nodus_shared::{AccountFeed, EventBatch, EventKind, InteractionEvent};
use url::Url;

/// The SDK as this app wires it: browser transports, memory cache, system clock.
///
/// The transports carry the token, so a session change rebuilds this value
/// instead of swapping a token underneath it.
pub type WebSdk = Sdk<BrowserInstance, BrowserDiscovery, MemoryPersistence, SystemClock>;

/// The error type [`WebSdk`] produces, for the one branch that inspects a variant.
type WebSdkError = nodus_sdk::SdkError<nodus_sdk::instance_api::Error, nodus_discovery::Error>;

/// `localStorage` key holding the milestone-1 session record.
const SESSION_KEY: &str = "nodus.session";

/// Where the API lives when the page has no `window` (native tests, prerender).
const FALLBACK_API_BASE: &str = "http://127.0.0.1:8080/api";

thread_local! {
    /// The one SDK instance for this app.
    static SDK: RefCell<Option<Rc<WebSdk>>> = const { RefCell::new(None) };
    /// The instance transport, kept separately: the SDK exposes no way to call
    /// `follow` (it is a state machine, not an action surface), and a follow is a
    /// plain instance write.
    static INSTANCE: RefCell<Option<BrowserInstance>> = const { RefCell::new(None) };
    /// The feed snapshot id of the last loaded page: `EventBatch` needs it and
    /// `FeedState` does not keep it.
    static FEED_ID: RefCell<Option<String>> = const { RefCell::new(None) };
    /// Request flags for the Feed tab; created with [`init`].
    static FEED_UI: RefCell<Option<FeedUi>> = const { RefCell::new(None) };
    /// Bumped by every SDK subscription; views `.get()` it to track changes.
    static REVISION: RefCell<Option<RwSignal<u64>>> = const { RefCell::new(None) };
    /// Subscriptions must outlive `init`; dropping them stops the bumps.
    static SUBSCRIPTIONS: RefCell<Vec<Subscription>> = const { RefCell::new(Vec::new()) };
}

/// The Feed tab's request state, which belongs to the view and not to the SDK.
#[derive(Debug, Clone, Copy)]
pub struct FeedUi {
    /// A feed request is in flight; the list stays visible, dimmed.
    pub refreshing: RwSignal<bool>,
    /// The last request's error, rendered inline with a retry.
    pub error: RwSignal<Option<String>>,
}

impl FeedUi {
    fn new() -> Self {
        Self {
            refreshing: RwSignal::new(false),
            error: RwSignal::new(None),
        }
    }
}

/// The Feed tab's request flags. Falls back to fresh signals if a view is built
/// before [`init`], so rendering order can never panic.
pub fn feed_ui() -> FeedUi {
    FEED_UI.with_borrow_mut(|slot| *slot.get_or_insert_with(FeedUi::new))
}

/// Wires the SDK into the reactive graph, then restores a stored session in the
/// background. Call once, from the root component.
///
/// Idempotent: a second call is a no-op, so remounting the app does not
/// double-subscribe.
pub fn init() {
    if REVISION.with_borrow(Option::is_some) {
        return;
    }

    let revision = RwSignal::new(0_u64);
    REVISION.with_borrow_mut(|slot| *slot = Some(revision));
    feed_ui();

    // Until a session exists the views only read state, but `with_sdk` must still
    // answer, so install an unauthenticated SDK for the API origin.
    if let Some(base) = api_base() {
        let _ = install(&Instance { base }, "");
    }

    let Some(stored) = load_session() else {
        return;
    };
    let Ok(instance) = Instance::parse(&stored.host) else {
        // A host we can no longer parse (or a hand-edited record) is not a
        // session; drop it rather than fail every boot.
        clear_session();
        return;
    };
    if install(&instance, &stored.token).is_err() {
        return;
    }
    restore(stored);
}

/// The revision signal: read it inside a view to re-render on SDK changes.
///
/// # Panics
///
/// Panics if [`init`] has not run, which would silently produce a view that never
/// updates — a loud failure is the safer one.
#[must_use]
pub fn revision() -> RwSignal<u64> {
    REVISION
        .with_borrow(|slot| *slot)
        .expect("state::bridge::init() must run before rendering views")
}

/// Reads SDK state. Read [`revision`] first, in the same reactive scope.
///
/// # Panics
///
/// Panics if [`init`] has not run.
pub fn with_sdk<R>(f: impl FnOnce(&WebSdk) -> R) -> R {
    SDK.with_borrow(|slot| {
        let sdk = slot
            .as_ref()
            .expect("state::bridge::init() must run before rendering views");
        f(sdk)
    })
}

/// Applies a command, triggering re-renders through [`revision`].
pub fn dispatch(command: Command) {
    if let Some(sdk) = handle() {
        sdk.dispatch(command);
    }
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

/// Loads one page of the feed, keeping the current list visible while it runs.
///
/// `cursor` is `None` for a refresh and the previous page's `next_cursor` for
/// "Load more".
pub fn reload_feed(cursor: Option<String>) {
    let Some(sdk) = handle() else {
        return;
    };
    let ui = feed_ui();
    ui.refreshing.set(true);
    ui.error.set(None);
    wasm_bindgen_futures::spawn_local(async move {
        match sdk.load_feed(cursor.as_deref()).await {
            Ok(feed) => remember_feed(&feed),
            Err(error) => ui.error.set(Some(error.to_string())),
        }
        ui.refreshing.set(false);
    });
}

/// Follows the account behind a feed item, then hides the item.
///
/// The follow is an instance write, not an SDK command, so it goes through the
/// transport; hiding the item is state, so it goes through the SDK. On error the
/// item stays and the caller shows the message (the button returns to "Follow").
///
/// # Errors
///
/// Returns a message fit for inline display; nothing is dismissed when the
/// instance call fails.
pub async fn follow_item(item_id: i64, account_id: &str) -> Result<(), String> {
    let transport = INSTANCE
        .with_borrow(Clone::clone)
        .ok_or_else(|| "Not signed in.".to_owned())?;
    transport
        .follow(&LocalId(account_id.to_owned()))
        .await
        .map_err(|error| error.to_string())?;
    dispatch(Command::DismissItem { item_id });
    Ok(())
}

/// Reports "not interested" for a feed item, then hides the item.
///
/// # Errors
///
/// Returns a message fit for inline display; the item is kept when the report
/// fails, so the user's action is never silently lost.
pub async fn not_interested(item_id: i64) -> Result<(), String> {
    let sdk = handle().ok_or_else(|| "Not signed in.".to_owned())?;
    let feed_id = FEED_ID
        .with_borrow(Clone::clone)
        .ok_or_else(|| "The feed has not loaded yet.".to_owned())?;

    let batch = EventBatch {
        feed_id,
        events: vec![InteractionEvent {
            item_id,
            kind: EventKind::NotInterested,
            client_ts: client_timestamp(),
            dwell_ms: None,
        }],
    };
    sdk.report_events(batch)
        .await
        .map_err(|error| error.to_string())?;
    dispatch(Command::DismissItem { item_id });
    Ok(())
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
    let acct = handle().map_or_else(String::new, |sdk| {
        sdk.state()
            .session
            .get()
            .map_or_else(String::new, |session| session.acct)
    });
    let record = StoredSession {
        host: host.to_owned(),
        acct,
        token: token.to_owned(),
    };
    let Ok(json) = serde_json::to_string(&record) else {
        return;
    };
    if let Some(storage) = local_storage() {
        let _ = storage.set_item(SESSION_KEY, &json);
    }
}

/// Forgets the stored session. Does not touch the SDK; see [`sign_out`] for that.
pub fn clear_session() {
    if let Some(storage) = local_storage() {
        let _ = storage.remove_item(SESSION_KEY);
    }
    FEED_ID.with_borrow_mut(|slot| *slot = None);
}

/// The `localStorage` record for a signed-in session.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct StoredSession {
    /// Instance host, as the user typed it.
    host: String,
    /// `user@host` of the account, for display before the first request returns.
    acct: String,
    /// OAuth bearer token for the instance and for the recommendation API.
    token: String,
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

/// Loads the newest feed page and clears the refresh flag.
async fn load_first_page(sdk: &WebSdk, ui: FeedUi) {
    match sdk.load_feed(None).await {
        Ok(feed) => remember_feed(&feed),
        Err(error) => ui.error.set(Some(error.to_string())),
    }
    ui.refreshing.set(false);
}

/// Keeps the snapshot id that event batches must carry.
fn remember_feed(feed: &AccountFeed) {
    FEED_ID.with_borrow_mut(|slot| *slot = Some(feed.feed_id.clone()));
    feed_ui().error.set(None);
}

/// Builds a [`WebSdk`] for `instance` and swaps it in. Replaces any previous one.
fn install(instance: &Instance, token: &str) -> Result<(), String> {
    let base = api_base()
        .ok_or_else(|| "Could not determine the API origin for this page.".to_owned())?;
    let transport = BrowserInstance::new(instance, token);
    let sdk = Rc::new(Sdk::new(
        transport.clone(),
        BrowserDiscovery::new(base, token),
        MemoryPersistence::new(),
        SystemClock,
    ));

    let revision = revision();
    let bump = move || revision.update(|generation| *generation += 1);
    let mut subscriptions = Vec::new();
    {
        let state = sdk.state();
        subscriptions.push(state.session.subscribe(bump));
        subscriptions.push(state.stream.subscribe(bump));
        subscriptions.push(state.feed.subscribe(bump));
        subscriptions.push(state.timelines.subscribe(bump));
    }

    // Dropping the previous vector unsubscribes from the previous SDK, before that
    // SDK is replaced, so no callback can fire against state nothing renders.
    SUBSCRIPTIONS.with_borrow_mut(|slot| *slot = subscriptions);
    INSTANCE.with_borrow_mut(|slot| *slot = Some(transport));
    SDK.with_borrow_mut(|slot| *slot = Some(sdk));
    revision.update(|generation| *generation += 1);
    Ok(())
}

/// The current SDK, if [`init`] has installed one.
fn handle() -> Option<Rc<WebSdk>> {
    SDK.with_borrow(std::clone::Clone::clone)
}

/// The API base for this page: this origin plus `/api`.
///
/// Both the Trunk dev proxy (`web/Trunk.toml`) and a same-origin deployment serve
/// the API under `/api`, so the origin is all the client needs to know. Built with
/// `web_sys::Url` rather than string formatting so a path-prefixed or port-bearing
/// origin cannot produce a double or missing slash.
fn api_base() -> Option<Url> {
    let from_window = web_sys::window().and_then(|window| {
        let origin = window.location().origin().ok()?;
        let url = web_sys::Url::new(&origin).ok()?;
        url.set_pathname("/api");
        Url::parse(&url.href()).ok()
    });
    from_window.or_else(|| Url::parse(FALLBACK_API_BASE).ok())
}

/// The scopes a pasted token is assumed to carry.
///
/// The instance's token page hands out `read write follow`; OAuth will let the
/// user narrow this, and PKCE will request exactly what the app uses.
fn scopes() -> Vec<String> {
    vec!["read".to_owned(), "write".to_owned(), "follow".to_owned()]
}

/// The stored session, if there is a well-formed one.
fn load_session() -> Option<StoredSession> {
    let raw = local_storage()?.get_item(SESSION_KEY).ok()??;
    serde_json::from_str(&raw).ok()
}

/// `localStorage`, when the browser allows it (it throws in some private modes).
fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

/// RFC 3339 timestamp from the browser clock; the server clamps it to ±10 minutes.
fn client_timestamp() -> String {
    js_sys::Date::new_0().to_iso_string().into()
}
