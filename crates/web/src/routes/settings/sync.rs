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

/// The card's own state, kept outside the component.
///
/// A route change or a state notification re-renders the screen, and a signal created
/// inside the component would start again from "no dialog": the modal would open and
/// close in the same frame, and the message explaining the result would vanish. These
/// three are the only things the card owns, so they live as long as the page does.
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

fn ui() -> SyncUi {
    UI.with(|ui| SyncUi {
        confirming: ui.confirming,
        busy: ui.busy,
        result: ui.result,
    })
}

/// The sync consent, with the button that turns it on or off.
#[component]
pub fn SyncCard() -> impl IntoView {
    let revision = revision();
    let ui = ui();
    let busy = ui.busy;
    let result = ui.result;
    let confirming = ui.confirming;

    // What the server holds, not what this tab remembers.
    let consent = move || {
        revision.get();
        with_sdk(|sdk| sdk.state().sync.get())
    };

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
        if consent().enabled {
            Intent::Disable
        } else {
            Intent::Enable
        }
    };

    view! {
        <div class="card border border-base-300 bg-base-100 shadow-sm">
            <div class="card-body gap-3 p-4">
                <h2 class="card-title text-base">"Background sync"</h2>
                <p class="text-sm opacity-80">
                    "Turning this on lets the recommendation server keep an encrypted copy of your access token, sync your follows in the background, and rank without waiting for you to open the app. Turning it off deletes that copy. Interaction telemetry is a separate consent and stays off."
                </p>
                <p class="flex items-center gap-2 text-sm">
                    <span
                        class=move || if consent().enabled {
                            "badge badge-success badge-sm"
                        } else {
                            "badge badge-ghost badge-sm"
                        }
                    >
                        {move || if consent().enabled { "on" } else { "off" }}
                    </span>
                    <span class="opacity-70">
                        {move || if consent().enabled {
                            "This account is synced by the server."
                        } else {
                            "Follows are only read while this app is open."
                        }}
                    </span>
                </p>
                <div class="card-actions justify-end">
                    <button
                        class=move || if consent().enabled {
                            "btn btn-ghost btn-sm"
                        } else {
                            "btn btn-primary btn-sm"
                        }
                        disabled=move || busy.get()
                        on:click=move |_| confirming.set(Some(asked()))
                    >
                        <Show when=move || busy.get()>
                            <span class="loading loading-spinner loading-xs"></span>
                        </Show>
                        {move || if consent().enabled {
                            "Turn off background sync"
                        } else {
                            "Enable background sync"
                        }}
                    </button>
                </div>
                {move || result.get().map(|message| view! {
                    <div class="alert alert-soft alert-info py-2 text-sm" role="status">
                        <span>{message}</span>
                    </div>
                })}
            </div>

            <ConfirmDialog confirming=confirming run=Callback::new(run)/>
        </div>
    }
}
