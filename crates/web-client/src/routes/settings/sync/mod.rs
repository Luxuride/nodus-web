//! The background-sync card: a consent the server holds, rendered as a toggle.
//!
//! Two rules make it honest. The label comes from `AppState::sync`, which is only
//! written by the server's answer (or by a successful `set_sync`), so a second
//! browser cannot leave this screen claiming "on" after the token was revoked
//! elsewhere. And the state changes only after the request succeeded: a toggle that
//! flips on a failed call claims a consent nobody granted.
//!
//! The rows and the dialog are shared; this file owns the reads, the confirmation
//! state and what a confirmed intent does.

use leptos::prelude::*;
use nodus_view::pages::settings::sync::{ConfirmDialog, SyncIntent, SyncRows};

use crate::state::alive::Alive;
use crate::state::bridge::{load_sync_consent, revision, with_sdk};

mod authorize;
mod decide;

/// The card's own state, created once, outside every component.
///
/// A `RwSignal` belongs to the owner that was current when it was created, and a route
/// change disposes that owner. Creating these lazily on first render therefore handed
/// the *second* visit to Settings three disposed handles, and reading one panicked with
/// "Tried to access a reactive value that has already been disposed" — inside a
/// `spawn_local` poll, which poisons the executor and freezes the page.
///
/// [`install`] runs from `main` before the first mount, where no owner exists, so these
/// signals have no owner and no navigation can dispose them.
struct SyncUi {
    confirming: RwSignal<Option<SyncIntent>>,
    busy: RwSignal<bool>,
    result: RwSignal<Option<String>>,
}

thread_local! {
    static UI: SyncUi = SyncUi {
        confirming: RwSignal::new(None),
        busy: RwSignal::new(false),
        result: RwSignal::new(None),
    };
}

/// Creates the card's signals while no component owner is current.
///
/// Must be called from `main`, before `mount_to_body`: called from a component, the
/// signals would belong to that component and die with it.
pub fn install() {
    UI.with(|_| ());
}

/// The shared handles, cheap to copy.
fn ui() -> (
    RwSignal<Option<SyncIntent>>,
    RwSignal<bool>,
    RwSignal<Option<String>>,
) {
    UI.with(|ui| (ui.confirming, ui.busy, ui.result))
}

/// The sync consent, with the button that turns it on or off.
#[component]
pub fn SyncCard() -> impl IntoView {
    let revision = revision();
    let (confirming, busy, result) = ui();

    // What the server holds, not what this tab remembers. `None` means it has not
    // answered yet, which is a third state and must not be rendered as "off".
    let state = Signal::derive(move || {
        revision.get();
        with_sdk(|sdk| sdk.state().sync.get().map(|consent| consent.enabled))
    });

    // Asked once per visit. Reading no signal keeps this from re-firing on every
    // state change, which would turn each answer into the next request.
    Effect::new(move |_| {
        wasm_bindgen_futures::spawn_local(async {
            let _ = load_sync_consent().await;
        });
    });

    let run = move |intent: SyncIntent| {
        decide::run(intent, confirming, result, busy, Alive::new());
    };

    // What the button asks for: the opposite of what the server currently holds.
    let asked = move || {
        if state.get() == Some(true) {
            SyncIntent::Disable
        } else {
            SyncIntent::Enable
        }
    };

    view! {
        <nodus_view::components::panel::Panel title="Background sync">
            <SyncRows
                state=state
                busy=Signal::derive(move || busy.get())
                result=Signal::derive(move || result.get())
                on_ask=Some(Callback::new(move |()| confirming.set(Some(asked()))))
            />
        </nodus_view::components::panel::Panel>

        <ConfirmDialog
            confirming=Signal::derive(move || confirming.get())
            on_cancel=Some(Callback::new(move |()| confirming.set(None)))
            on_confirm=Some(Callback::new(move |intent: SyncIntent| run(intent)))
        />
    }
}
