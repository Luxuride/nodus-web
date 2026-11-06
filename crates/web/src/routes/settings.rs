//! Settings: the instance this browser holds a session for, the sync consent, and
//! sign-out.
//!
//! The sync toggle is a stub in the sense that there is no "on" switch yet — it
//! makes the one call the contract has (`set_sync(true, false)`) and renders what
//! came back, so the consent copy and the failure path exist before the feature.

use leptos::prelude::*;

use crate::state::bridge::{revision, set_sync, sign_out, with_sdk};

/// The settings screen.
#[component]
pub fn Settings() -> impl IntoView {
    let revision = revision();

    let host = move || {
        revision.get();
        with_sdk(|sdk| {
            sdk.state()
                .session
                .get()
                .map_or_else(String::new, |session| session.host)
        })
    };
    let acct = move || {
        revision.get();
        with_sdk(|sdk| {
            sdk.state()
                .session
                .get()
                .map_or_else(String::new, |session| session.acct)
        })
    };

    let enabling = RwSignal::new(false);
    let sync_result = RwSignal::new(None::<String>);

    let on_enable_sync = move |_| {
        if enabling.get_untracked() {
            return;
        }
        enabling.set(true);
        sync_result.set(None);
        wasm_bindgen_futures::spawn_local(async move {
            let message = match set_sync(true, false).await {
                Ok(()) => "Background sync is on. The server keeps an encrypted copy of your \
                           token and syncs your follows for you."
                    .to_owned(),
                Err(error) => format!("Could not enable background sync: {error}"),
            };
            sync_result.set(Some(message));
            enabling.set(false);
        });
    };

    view! {
        <section class="mx-auto w-full max-w-2xl px-4 py-6">
            <h1 class="mb-5 text-xl font-semibold tracking-tight">"Settings"</h1>

            <div class="card mb-4 border border-base-300 bg-base-100 shadow-sm">
                <div class="card-body gap-2 p-4">
                    <h2 class="card-title text-base">"Session"</h2>
                    <dl class="grid grid-cols-[8rem_minmax(0,1fr)] gap-x-3 gap-y-1 text-sm">
                        <dt class="opacity-60">"Instance"</dt>
                        <dd class="truncate">{move || host()}</dd>
                        <dt class="opacity-60">"Account"</dt>
                        <dd class="truncate">{move || acct()}</dd>
                    </dl>
                    <div class="card-actions mt-2 justify-end">
                        <button class="btn btn-ghost btn-sm" on:click=move |_| sign_out()>
                            "Sign out"
                        </button>
                    </div>
                    <p class="text-xs opacity-60">"Signing out forgets the token stored in this browser."</p>
                </div>
            </div>

            <div class="card border border-base-300 bg-base-100 shadow-sm">
                <div class="card-body gap-3 p-4">
                    <h2 class="card-title text-base">"Background sync"</h2>
                    <p class="text-sm opacity-80">
                        "Turning this on uploads your access token to the recommendation server so it can sync your follows in the background and rank without waiting for you to open the app. Interaction telemetry is a separate consent and stays off."
                    </p>
                    <div class="card-actions justify-end">
                        <button
                            class="btn btn-primary btn-sm"
                            disabled=move || enabling.get()
                            on:click=on_enable_sync
                        >
                            <Show when=move || enabling.get()>
                                <span class="loading loading-spinner loading-xs"></span>
                            </Show>
                            "Enable background sync"
                        </button>
                    </div>
                    {move || sync_result.get().map(|message| view! {
                        <div class="alert alert-soft alert-info py-2 text-sm" role="status">
                            <span>{message}</span>
                        </div>
                    })}
                </div>
            </div>
        </section>
    }
}
