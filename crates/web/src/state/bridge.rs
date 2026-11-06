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
//! See `web/docs/02-architecture.md` ("SDK owns state") and `sdk/README.md`.

use std::cell::RefCell;

use leptos::prelude::*;
use mastodon_reccom_sdk::{Command, ShellSdk, Subscription, shell_sdk};

thread_local! {
    /// The one SDK instance for this app.
    static SDK: RefCell<Option<ShellSdk>> = const { RefCell::new(None) };
    /// Bumped by every SDK subscription; views `.get()` it to track changes.
    static REVISION: RefCell<Option<RwSignal<u64>>> = const { RefCell::new(None) };
    /// Subscriptions must outlive `init`; dropping them stops the bumps.
    static SUBSCRIPTIONS: RefCell<Vec<Subscription>> = const { RefCell::new(Vec::new()) };
}

/// Wires the SDK into the reactive graph. Call once, from the root component.
///
/// Idempotent: a second call is a no-op, so remounting the app does not
/// double-subscribe.
pub fn init() {
    if REVISION.with_borrow(Option::is_some) {
        return;
    }

    let revision = RwSignal::new(0_u64);
    REVISION.with_borrow_mut(|slot| *slot = Some(revision));

    let sdk = shell_sdk();
    let mut subscriptions = Vec::new();
    {
        let state = sdk.state();
        let bump = move || revision.update(|generation| *generation += 1);
        subscriptions.push(state.session.subscribe(bump));
        subscriptions.push(state.stream.subscribe(bump));
        subscriptions.push(state.feed.subscribe(bump));
        subscriptions.push(state.timelines.subscribe(bump));
    }

    SUBSCRIPTIONS.with_borrow_mut(|slot| *slot = subscriptions);
    SDK.with_borrow_mut(|slot| *slot = Some(sdk));
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
pub fn with_sdk<R>(f: impl FnOnce(&ShellSdk) -> R) -> R {
    SDK.with_borrow(|slot| {
        let sdk = slot
            .as_ref()
            .expect("state::bridge::init() must run before rendering views");
        f(sdk)
    })
}

/// Applies a command, triggering re-renders through [`revision`].
#[expect(
    dead_code,
    reason = "the Feed view is the first dispatcher; it lands with M1"
)]
pub fn dispatch(command: Command) {
    with_sdk(|sdk| sdk.dispatch(command));
}
