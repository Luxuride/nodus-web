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
use nodus_sdk::instance_api::Instance;
use nodus_sdk::{Command, MemoryPersistence, Sdk, Subscription, SystemClock};
use url::Url;

mod feed;
mod session;

// Re-exported so every call site keeps saying `bridge::<verb>`: the split into
// files is a reading aid, not a new API surface.
pub use feed::{follow_item, not_interested, reload_feed, unfollow_item};
pub use session::{set_sync, sign_in, sign_out};

/// The SDK as this app wires it: browser transports, memory cache, system clock.
///
/// The transports carry the token, so a session change rebuilds this value
/// instead of swapping a token underneath it.
pub type WebSdk = Sdk<BrowserInstance, BrowserDiscovery, MemoryPersistence, SystemClock>;

/// The error type [`WebSdk`] produces, for the one branch that inspects a variant.
type WebSdkError = nodus_sdk::SdkError<nodus_sdk::instance_api::Error, nodus_discovery::Error>;

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

    // Reading, validating and re-installing the stored record is the session
    // module's job; `init` only asks for it once the unauthenticated SDK exists.
    session::restore_saved();
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

/// Drops the snapshot id kept for event batches.
///
/// Called when the session ends, so a late batch cannot carry the previous
/// account's snapshot id into the next one.
pub(super) fn forget_feed_id() {
    FEED_ID.with_borrow_mut(|slot| *slot = None);
}

/// Builds a [`WebSdk`] for `instance` and swaps it in. Replaces any previous one.
fn install(instance: &Instance, token: &str) -> Result<(), String> {
    let base =
        api_base().ok_or_else(|| "Could not determine the API origin for this page.".to_owned())?;
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

/// The instance transport, when a session has installed one.
///
/// Follow and unfollow are actions, not state, so they cannot go through
/// `dispatch`, and the SDK exposes no action surface: the transport is kept beside
/// it and handed to the [`feed`] verbs instead.
fn instance_transport() -> Option<BrowserInstance> {
    INSTANCE.with_borrow(Clone::clone)
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
