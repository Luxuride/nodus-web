//! Sign-in: paste an instance host and an access token.
//!
//! This is deliberately the smallest thing that works. There is no OAuth redirect
//! yet: the user creates a token on their instance and pastes it, and the app
//! verifies it before it saves or uses anything. Browser sign-in (dynamic client
//! registration, PKCE, revocable tokens in `IndexedDB`) is the next milestone, per
//! `web/docs/03-data-layer.md`.

use leptos::ev::SubmitEvent;
use leptos::prelude::*;

use crate::state::bridge::{feed_ui, sign_in};

/// The sign-in screen.
#[component]
pub fn SignIn() -> impl IntoView {
    let host = RwSignal::new(String::new());
    let token = RwSignal::new(String::new());
    let error = RwSignal::new(None::<String>);
    let busy = RwSignal::new(false);
    // A stored session that failed to restore explains itself here, where the user
    // can act on it.
    let startup_error = feed_ui().error;

    let submit = move |event: SubmitEvent| {
        event.prevent_default();
        if busy.get_untracked() {
            return;
        }
        busy.set(true);
        error.set(None);
        let host = host.get_untracked();
        let token = token.get_untracked();
        wasm_bindgen_futures::spawn_local(async move {
            match sign_in(&host, &token).await {
                Ok(()) => {}
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    view! {
        <section class="mx-auto w-full max-w-md px-4 py-10">
            <div class="card border border-base-300 bg-base-100 shadow-sm">
                <div class="card-body gap-4">
                    <h1 class="card-title">"Sign in"</h1>
                    <p class="text-sm opacity-70">
                        "Paste a token from your instance for now; browser sign-in lands with the PKCE milestone."
                    </p>
                    <form class="flex flex-col gap-3" on:submit=submit>
                        <label class="form-control w-full">
                            <span class="label-text mb-1 text-sm">"Instance"</span>
                            <input
                                class="input input-bordered w-full"
                                type="text"
                                placeholder="mastodon.social"
                                autocomplete="url"
                                prop:value=move || host.get()
                                on:input=move |event| host.set(event_target_value(&event))
                                disabled=move || busy.get()
                            />
                        </label>
                        <label class="form-control w-full">
                            <span class="label-text mb-1 text-sm">"Access token"</span>
                            <input
                                class="input input-bordered w-full"
                                type="password"
                                autocomplete="off"
                                prop:value=move || token.get()
                                on:input=move |event| token.set(event_target_value(&event))
                                disabled=move || busy.get()
                            />
                        </label>
                        <button
                            class="btn btn-primary mt-2"
                            type="submit"
                            disabled=move || busy.get()
                        >
                            <Show when=move || busy.get()>
                                <span class="loading loading-spinner loading-xs"></span>
                            </Show>
                            "Sign in"
                        </button>
                    </form>
                    {move || error.get().or_else(|| startup_error.get()).map(|message| view! {
                        <div class="alert alert-soft alert-error text-sm" role="alert">
                            <span>{message}</span>
                        </div>
                    })}
                </div>
            </div>
        </section>
    }
}
