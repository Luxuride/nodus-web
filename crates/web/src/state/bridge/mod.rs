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
use nodus_instance::BrowserInstance;
use nodus_sdk::instance_api::Instance;
use nodus_sdk::{Command, MemoryPersistence, Sdk, Subscription, SystemClock};
use url::Url;

pub mod accounts;
pub mod actions;
pub mod admin;
mod admin_token;
mod feed;
mod impressions;
mod posts;
mod runtime;
pub mod session;
pub mod notices;
pub mod pages;
pub mod timeline;

// Re-exported so every call site keeps saying `bridge::<verb>`: the split into
// files is a reading aid, not a new API surface.
pub use feed::{follow_item, not_interested, reload_feed, unfollow_item};
pub use impressions::{
    begin as begin_impression, finish as finish_impression, install as install_impression_flush,
};
pub use posts::{load_posts_once, posts_ui, reload_posts};
pub use runtime::start as start_background_tick;
pub use session::{authorize_sync, load_sync_consent, set_sync, sign_in, sign_out};

/// The SDK as this app wires it: browser transports, memory cache, system clock.
///
/// The transports carry the token, so a session change rebuilds this value
/// instead of swapping a token underneath it.
pub type WebSdk = Sdk<BrowserInstance, BrowserDiscovery, MemoryPersistence, SystemClock>;

/// The error type [`WebSdk`] produces, for the one branch that inspects a variant.
type WebSdkError = nodus_sdk::SdkError<nodus_sdk::instance_api::Error, nodus_discovery::Error>;

/// Where the API lives when the page has no `window` (native tests, prerender).
const FALLBACK_API_BASE: &str = "http://127.0.0.1:3000";

/// One account's SDK, transport and subscriptions.
struct Live {
    sdk: Rc<WebSdk>,
    transport: BrowserInstance,
    /// Kept alive so this account keeps receiving its own events; dropping the SDK
    /// while these lived on would call back into a store nothing renders. Never read,
    /// which is the point of an owning field.
    _subscriptions: Vec<Subscription>,
}

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
    /// Whether [`init`] has wired the SDK, so a remount does not subscribe twice.
    static INSTALLED: core::cell::Cell<bool> = const { core::cell::Cell::new(false) };
    /// One installed SDK per account, so switching does not rebuild state (and does not
    /// re-fetch a timeline a previous visit already holds). Subscriptions live here too:
    /// an inactive account still receives socket frames for its own sessions, and those
    /// frames land in its own store.
    static LIVE: RefCell<std::collections::HashMap<String, Live>> = RefCell::new(std::collections::HashMap::new());
    /// The SDK built by the last [`install`], before it has an account to belong to.
    ///
    /// The host and token arrive before the session does — `sign_in` is what learns the
    /// account id — so the SDK waits here until [`register`] can name it.
    static PENDING: RefCell<Option<Live>> = const { RefCell::new(None) };
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
    if INSTALLED.get() {
        return;
    }
    INSTALLED.set(true);
    install_reactive_root();

    // Until a session exists the views only read state, but `with_sdk` must still
    // answer, so install an unauthenticated SDK for the API origin.
    if let Some(base) = api_base() {
        let _ = install(&Instance { base }, "");
    }

    // Reading, validating and re-installing the stored record is the session
    // module's job; `init` only asks for it once the unauthenticated SDK exists.
    session::restore_saved();
}

/// Creates the app-lifetime signals while no owner exists.
///
/// Must run before the first mount. A signal created inside a component is owned by it,
/// so a later write from a socket frame or a tick — both outlive the screen that
/// happened to be mounted first — would raise
/// `reactive_graph … Tried to access a reactive value that has already been disposed`,
/// panic inside a task poll, and leave the executor unable to resume anything.
pub fn install_reactive_root() {
    if REVISION.with_borrow(Option::is_none) {
        let revision = RwSignal::new(0_u64);
        REVISION.with_borrow_mut(|slot| *slot = Some(revision));
    }
    feed_ui();
    posts::install();
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

/// The snapshot id of the list currently on screen, if any.
///
/// A refresh passes it to the server so the request is understood as superseding that
/// snapshot; a "load more" never does, because it is a read of the same list.
pub(super) fn existing_feed_id() -> Option<String> {
    FEED_ID.with_borrow(std::clone::Clone::clone)
}

/// Drops the snapshot id kept for event batches.
///
/// Called when the session ends, so a late batch cannot carry the previous
/// account's snapshot id into the next one.
pub(super) fn forget_feed_id() {
    FEED_ID.with_borrow_mut(|slot| *slot = None);
}

/// Builds a [`WebSdk`] for `instance` and swaps it in. Replaces any previous one.
use nodus_instance::actions::AdminActions;

fn install(instance: &Instance, token: &str) -> Result<(), String> {
    let base =
        api_base().ok_or_else(|| "Could not determine the API origin for this page.".to_owned())?;
    let transport = BrowserInstance::new(instance, token);
    let sdk = Rc::new(Sdk::new(
        transport.clone(),
        BrowserDiscovery::new(base, instance.base.host_str().unwrap_or_default(), token),
        MemoryPersistence::new(),
        SystemClock,
    ));

    let revision = revision();
    // `try_update`, not `update`: this callback runs from socket frames and background
    // ticks, which outlive any single screen. A disposed signal must be a missed repaint,
    // never a panic inside a task poll.
    let bump = move || {
        let _ = revision.try_update(|generation| *generation += 1);
    };
    let mut subscriptions = Vec::new();
    {
        let state = sdk.state();
        subscriptions.push(state.capabilities.subscribe(bump));
        subscriptions.push(state.session.subscribe(bump));
        subscriptions.push(state.stream.subscribe(bump));
        subscriptions.push(state.feed.subscribe(bump));
        subscriptions.push(state.timelines.subscribe(bump));
        // Entities, for the same reason as pages: a status that arrives on the socket,
        // or is favourited in another tab, must repaint every screen showing it.
        subscriptions.push(state.statuses.subscribe(bump));
        subscriptions.push(state.accounts.subscribe(bump));
        // `sync` is a consent the *server* holds and the card's button is the only
        // writer, so without this subscription the toggle kept rendering the value it
        // read on mount until the operator reloaded the page.
        subscriptions.push(state.sync.subscribe(bump));
    }

    // Dropping the previous vector unsubscribes from the previous SDK, before that
    // SDK is replaced, so no callback can fire against state nothing renders.
    // Every socket frame is applied by the SDK before a view sees it: an event that
    // arrived on the wire has to land in the same store a page reads, or the screen
    // holding the status cannot repaint from it.
    let sink = wasm_bindgen::closure::Closure::<dyn FnMut(wasm_bindgen::JsValue)>::new({
        let sdk = Rc::clone(&sdk);
        move |frame: wasm_bindgen::JsValue| {
            if let Some(text) = frame.as_string() {
                sdk.apply_frame(&text);
            }
        }
    });
    let transport =
        transport.with_stream_sink(wasm_bindgen::JsCast::unchecked_into(sink.into_js_value()));

    SUBSCRIPTIONS.with_borrow_mut(|slot| *slot = Vec::new());
    INSTANCE.with_borrow_mut(|slot| *slot = Some(transport.clone()));
    PENDING.with_borrow_mut(|slot| {
        *slot = Some(Live {
            sdk: Rc::clone(&sdk),
            transport,
            _subscriptions: subscriptions,
        });
    });

    // Negotiate the protocol once per session. This is a question about the server,
    // not about a screen, so it must not be asked from a render: every view reads the
    // recorded answer and picks its path synchronously. A server that does not answer
    // keeps the Mastodon surface, which is the answer when there is no answer.
    wasm_bindgen_futures::spawn_local({
        let sdk = Rc::clone(&sdk);
        async move {
            if let Err(error) = sdk.load_capabilities().await {
                web_sys::console::warn_1(&format!("capabilities unavailable: {error}").into());
            }
        }
    });

    SDK.with_borrow_mut(|slot| *slot = Some(sdk));
    revision.update(|generation| *generation += 1);
    Ok(())
}

/// Names the SDK built by [`install`] and keeps it for switching.
///
/// Called once the session is known: the host and token arrive first, the account id
/// only when the instance answers, and an account the switcher cannot name is an
/// account the user cannot return to.
pub fn register(account: accounts::Account) {
    let key = account.key();
    let live = PENDING.with_borrow_mut(Option::take);
    if let Some(live) = live {
        LIVE.with_borrow_mut(|all| {
            all.insert(key.clone(), live);
        });
    }
    let mut list = accounts::load();
    accounts::insert(&mut list, account);
    accounts::store(&list);
    accounts::set_active(Some(&key));
    revision().update(|generation| *generation += 1);
}

/// The accounts this browser holds, oldest first.
#[must_use]
pub fn account_list() -> Vec<accounts::Account> {
    accounts::load()
}

/// The key of the account on screen.
#[must_use]
pub fn active_key() -> Option<String> {
    accounts::active()
}

/// Show another account without signing out of this one.
///
/// The SDK is already built, so switching is a pointer move: the timeline the previous
/// visit loaded is exactly what the SDK should keep holding.
///
/// # Errors
///
/// Returns a message when the account has no installed SDK, which happens after a
/// reload: records are on disk, SDKs are not, and the caller must reinstall instead.
pub fn switch(key: &str) -> Result<(), String> {
    let live = LIVE.with_borrow(|all| {
        all.get(key)
            .map(|live| (Rc::clone(&live.sdk), live.transport.clone()))
    });
    let Some((sdk, transport)) = live else {
        return Err("That account needs its session restored first.".to_owned());
    };
    INSTANCE.with_borrow_mut(|slot| *slot = Some(transport));
    SDK.with_borrow_mut(|slot| *slot = Some(sdk));
    accounts::set_active(Some(key));
    forget_feed_id();
    revision().update(|generation| *generation += 1);
    Ok(())
}

/// Forget one account, and fall back to another if it was the one on screen.
pub fn forget(key: &str) {
    LIVE.with_borrow_mut(|all| {
        all.remove(key);
    });
    let mut list = accounts::load();
    list.retain(|account| account.key() != key);
    accounts::store(&list);
    if active_key().as_deref() == Some(key) {
        match list.first() {
            Some(next) if switch(&next.key()).is_ok() => {}
            _ => {
                accounts::set_active(None);
                SDK.with_borrow_mut(|slot| *slot = None);
                INSTANCE.with_borrow_mut(|slot| *slot = None);
                revision().update(|generation| *generation += 1);
            }
        }
    }
}

/// The current SDK, if [`init`] has installed one.
fn handle() -> Option<Rc<WebSdk>> {
    SDK.with_borrow(std::clone::Clone::clone)
}

/// Keeps an admin grant for `host`, after proving the token can moderate.
///
/// The probe is the point: a token that cannot open the report queue would leave the
/// Administration tab claiming a permission it does not have, and the operator would
/// see an empty screen instead of "the grant did not work".
pub async fn keep_admin_token(host: &str, token: &str) -> Result<(), String> {
    let instance = Instance::parse(host).map_err(|error| error.to_string())?;
    let transport = BrowserInstance::new(&instance, token);
    transport
        .reports(nodus_instance::model::ReportState::Open, 1)
        .await
        .map_err(|error| {
            format!(
                "That account cannot moderate on {host}, so the grant was not kept: {error}"
            )
        })?;
    admin_token::store(host, token);
    Ok(())
}

/// The admin transport for the signed-in instance, when a grant was kept.
///
/// Moderation calls go through this one; everything else keeps the ordinary session
/// transport, which is what makes a leaked grant a narrower loss than a leaked session.
pub fn admin_transport() -> Option<BrowserInstance> {
    let host = with_sdk(|sdk| sdk.state().session.get().map(|session| session.host))?;
    let token = admin_token::load(&host)?;
    let base = api_base()?;
    let _ = base;
    let instance = Instance::parse(&host).ok()?;
    Some(BrowserInstance::new(&instance, &token))
}

/// The instance whose grant would be used, if any is held.
pub fn admin_host() -> Option<String> {
    with_sdk(|sdk| sdk.state().session.get().map(|session| session.host))
}

/// Forgets the admin grant for the signed-in instance.
pub fn forget_admin_token() {
    if let Some(host) = admin_host() {
        admin_token::clear(&host);
    }
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
/// The origin is the whole answer: every discovery path already begins with `/api/v1`,
/// so a base that carried `/api` would produce `/api/api/v1/...`.
///
/// The `/api` prefix belongs to the deployment, not to the client: Trunk proxies it to
/// the feeds service in development (`web/Trunk.toml`), and a same-origin deployment
/// lets Caddy serve `/api/*` from that service directly. Built with `web_sys::Url`
/// rather than string formatting so a port-bearing origin cannot produce a wrong slash.
fn api_base() -> Option<Url> {
    let from_window = web_sys::window().and_then(|window| {
        let origin = window.location().origin().ok()?;
        Url::parse(&origin).ok()
    });
    from_window.or_else(|| Url::parse(FALLBACK_API_BASE).ok())
}
