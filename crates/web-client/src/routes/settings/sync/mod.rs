//! The background-sync card: a consent the server holds, rendered as a toggle.
//!
//! Two rules make it honest. The label comes from `AppState::sync`, which is only
//! written by the server's answer (or by a successful `set_sync`), so a second
//! browser cannot leave this screen claiming "on" after the token was revoked
//! elsewhere. And the state changes only after the request succeeded: a toggle that
//! flips on a failed call claims a consent nobody granted.
//!
//! Both directions ask first. Enabling starts the server processing an account's
//! data; disabling deletes the copy it holds. Neither is a toggle-sized decision, so
//! the button opens a confirmation that says exactly what will happen, and the modal
//! is a `modal-open` div rather than a `<dialog>` so it needs no imperative
//! `show_modal()` call and re-renders with the rest of the view.

use leptos::prelude::*;

use self::confirm::{ConfirmDialog, Intent};
use crate::state::alive::Alive;
use crate::state::bridge::{load_sync_consent, revision, with_sdk};

mod authorize;
mod confirm;
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
    confirming: RwSignal<Option<Intent>>,
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
    RwSignal<Option<Intent>>,
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
    let consent = move || {
        revision.get();
        with_sdk(|sdk| sdk.state().sync.get())
    };
    let state = move || consent().map(|consent| consent.enabled);

    // Asked once per visit. Reading no signal keeps this from re-firing on every
    // state change, which would turn each answer into the next request.
    Effect::new(move |_| {
        wasm_bindgen_futures::spawn_local(async {
            let _ = load_sync_consent().await;
        });
    });

    let run = move |intent: Intent| {
        decide::run(intent, confirming, result, busy, Alive::new());
    };

    // What the button asks for: the opposite of what the server currently holds.
    let asked = move || {
        if state() == Some(true) {
            Intent::Disable
        } else {
            Intent::Enable
        }
    };

    view! {
        <nodus_view::components::panel::Panel title="Background sync">
                <p class="text-sm opacity-80">
                    "Turning this on lets the recommendation server keep an encrypted copy of your access token, sync your follows in the background, and rank without waiting for you to open the app. Turning it off deletes that copy. Interaction telemetry is a separate consent and stays off."
                </p>
                <p class="flex items-center gap-2 text-sm">
                    <span
                        class=move || match state() {
                            Some(true) => "badge badge-success badge-sm",
                            Some(false) => "badge badge-ghost badge-sm",
                            None => "badge badge-ghost badge-sm opacity-50",
                        }
                    >
                        // Not "off" until the server says so: an unknown consent is
                        // unknown, and showing a default makes the page load look like
                        // a decision the reader just made.
                        {move || match state() {
                            Some(true) => "on",
                            Some(false) => "off",
                            None => "…",
                        }}
                    </span>
                    <span class="opacity-70">
                        {move || match state() {
                            Some(true) => "This account is synced by the server.",
                            Some(false) => "Follows are only read while this app is open.",
                            None => "Asking the server what it holds…",
                        }}
                    </span>
                </p>
                <div class="card-actions justify-end">
                    <button
                        class=move || if state() == Some(true) {
                            "btn btn-ghost btn-sm"
                        } else {
                            "btn btn-primary btn-sm"
                        }
                        // A button that acts on a guess is worse than no button.
                        disabled=move || state().is_none() || busy.get()
                        on:click=move |_| confirming.set(Some(asked()))
                    >
                        <Show when=move || busy.get()>
                            <span class="loading loading-spinner loading-xs"></span>
                        </Show>
                        {move || match state() {
                            Some(true) => "Turn off background sync",
                            Some(false) => "Enable background sync",
                            None => "Background sync",
                        }}
                    </button>
                </div>
                {move || result.get().map(|message| view! {
                    <div class="alert alert-soft alert-info py-2 text-sm" role="status">
                        <span>{message}</span>
                    </div>
                })}
        </nodus_view::components::panel::Panel>

        <ConfirmDialog confirming=confirming run=Callback::new(run)/>
    }
}
