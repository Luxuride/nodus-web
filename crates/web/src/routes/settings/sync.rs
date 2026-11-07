//! The background-sync card: a consent the server holds, rendered as a toggle.
//!
//! Two rules make it honest. The label comes from `AppState::sync`, which is only
//! written by the server's answer (or by a successful `set_sync`), so a second
//! browser cannot leave this screen claiming "on" after the token was revoked
//! elsewhere. And the state changes only after the request succeeded: a toggle that
//! flips on a failed call claims a consent nobody granted.

use leptos::prelude::*;

use crate::state::alive::Alive;
use crate::state::bridge::{load_sync_consent, revision, set_sync, with_sdk};

/// The sync consent, with the button that turns it on or off.
#[component]
pub fn SyncCard() -> impl IntoView {
    let revision = revision();
    let busy = RwSignal::new(false);
    let result = RwSignal::new(None::<String>);

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

    let on_toggle = move |_| {
        if busy.get_untracked() {
            return;
        }
        busy.set(true);
        result.set(None);
        let enable = !consent().enabled;
        let alive = Alive::new();
        wasm_bindgen_futures::spawn_local(async move {
            let message = match set_sync(enable, false).await {
                Ok(()) if enable => "Background sync is on. The server keeps an encrypted copy \
                                     of your token and syncs your follows for you."
                    .to_owned(),
                Ok(()) => "Background sync is off, and the server deleted its copy of your token."
                    .to_owned(),
                Err(error) if enable => format!("Could not enable background sync: {error}"),
                Err(error) => format!("Could not turn background sync off: {error}"),
            };
            alive.if_alive(|| {
                result.set(Some(message));
                busy.set(false);
            });
        });
    };

    view! {
        <div class="card border border-base-300 bg-base-100 shadow-sm">
            <div class="card-body gap-3 p-4">
                <h2 class="card-title text-base">"Background sync"</h2>
                <p class="text-sm opacity-80">
                    "Turning this on uploads your access token to the recommendation server so it can sync your follows in the background and rank without waiting for you to open the app. Interaction telemetry is a separate consent and stays off."
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
                        on:click=on_toggle
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
        </div>
    }
}
