//! Sign-in: authorize against the instance, with token pasting as the fallback.
//!
//! The primary path is the browser flow in [`crate::state::oauth`]: register the
//! client, send the browser to the instance, come back through `/oauth/callback`.
//! The user never types a credential here — the instance's own consent page is the
//! only place a password is entered, and this app never sees it.
//!
//! The token form underneath is the escape hatch for instances whose authorization
//! endpoint this client cannot use (older or unusual forks, http-only deployments,
//! locked-down instances that refuse app registration). It is kept small and folded
//! away so it cannot be mistaken for the normal path.

use leptos::ev::SubmitEvent;
use leptos::prelude::*;

use crate::state::alive::Alive;
use crate::state::bridge::{feed_ui, sign_in};
use crate::state::oauth;

/// The sign-in screen.
///
/// `note` says what the user was on their way to. Without it, clicking a rail link
/// while signed out looks like a dead button: every gated route renders this same
/// screen, so the address bar changes and nothing else appears to.
#[component]
pub fn SignIn(#[prop(optional)] note: Option<&'static str>) -> impl IntoView {
    // One signal for the instance: both forms point at the same instance, and an
    // address typed into either field is what the other submits.
    let host = RwSignal::new(String::new());
    let token = RwSignal::new(String::new());
    let authorize_error = RwSignal::new(None::<String>);
    let token_error = RwSignal::new(None::<String>);
    let authorizing = RwSignal::new(false);
    let pasting = RwSignal::new(false);
    // A stored session that failed to restore explains itself here, where the user
    // can act on it.
    let startup_error = feed_ui().error;

    let authorize = move |event: SubmitEvent| {
        event.prevent_default();
        if authorizing.get_untracked() {
            return;
        }
        authorizing.set(true);
        authorize_error.set(None);
        let host = host.get_untracked();
        let alive = Alive::new();
        wasm_bindgen_futures::spawn_local(async move {
            // `Ok` means the browser is already navigating to the instance, so the
            // spinner deliberately stays on: putting the button back would invite a
            // second authorization request while the first is in flight.
            if let Err(message) = oauth::begin(&host).await {
                alive.if_alive(|| {
                    authorize_error.set(Some(message));
                    authorizing.set(false);
                });
            }
        });
    };

    view! {
        <nodus_view::Screen width=nodus_view::Width::Narrow>
            <div class="card border border-base-300 bg-base-100 shadow-sm">
                <div class="card-body gap-4">
                    <h1 class="card-title">"Sign in"</h1>
                    {note.map(|note| view! {
                        <p class="text-sm font-medium">{note}</p>
                    })}
                    <p class="text-sm opacity-70">
                        "Authorize this app on your instance. Approval happens on your instance's own page, and the access token it returns can be revoked there at any time."
                    </p>
                    <form class="flex flex-col gap-3" on:submit=authorize>
                        <label class="form-control w-full">
                            <span class="label-text mb-1 text-sm">"Instance"</span>
                            <input
                                class="input input-bordered w-full"
                                type="text"
                                placeholder="mastodon.social"
                                autocomplete="url"
                                prop:value=move || host.get()
                                on:input=move |event| host.set(event_target_value(&event))
                                disabled=move || authorizing.get() || pasting.get()
                            />
                        </label>
                        <button
                            class="btn btn-primary mt-2"
                            type="submit"
                            disabled=move || authorizing.get() || pasting.get()
                        >
                            <Show when=move || authorizing.get()>
                                <span class="loading loading-spinner loading-xs"></span>
                            </Show>
                            "Authorize"
                        </button>
                    </form>
                    {move || authorize_error.get().or_else(|| startup_error.get()).map(|message| view! {
                        <div class="alert alert-soft alert-error text-sm" role="alert">
                            <span>{message}</span>
                        </div>
                    })}
                    <TokenEscapeHatch
                        host=host
                        token=token
                        error=token_error
                        authorizing=authorizing
                        pasting=pasting
                    />
                </div>
            </div>
        </nodus_view::Screen>
    }
}

/// The token form, folded away under a disclosure.
///
/// Split out of [`SignIn`] so the primary path stays readable: a reader of the
/// main view should not have to scroll past the escape hatch to understand what
/// signing in normally does.
#[component]
fn TokenEscapeHatch(
    host: RwSignal<String>,
    token: RwSignal<String>,
    error: RwSignal<Option<String>>,
    authorizing: RwSignal<bool>,
    pasting: RwSignal<bool>,
) -> impl IntoView {
    let submit = move |event: SubmitEvent| {
        event.prevent_default();
        if pasting.get_untracked() {
            return;
        }
        pasting.set(true);
        error.set(None);
        let host = host.get_untracked();
        let token = token.get_untracked();
        let alive = Alive::new();
        wasm_bindgen_futures::spawn_local(async move {
            if let Err(message) = sign_in(&host, &token).await {
                alive.if_alive(|| error.set(Some(message)));
            }
            alive.if_alive(|| pasting.set(false));
        });
    };

    view! {
        <details class="mt-2 border-t border-base-300 pt-3">
            <summary class="cursor-pointer text-sm opacity-70">
                "Advanced: sign in with an access token"
            </summary>
            <p class="mt-2 text-xs opacity-60">
                "For an instance that does not support browser sign-in. Create a token in your instance's development settings with the read, write and follow scopes, then paste it here."
            </p>
            <form class="mt-2 flex flex-col gap-3" on:submit=submit>
                <label class="form-control w-full">
                    <span class="label-text mb-1 text-sm">"Instance"</span>
                    <input
                        class="input input-bordered input-sm w-full"
                        type="text"
                        placeholder="mastodon.social"
                        autocomplete="url"
                        prop:value=move || host.get()
                        on:input=move |event| host.set(event_target_value(&event))
                        disabled=move || authorizing.get() || pasting.get()
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
                        disabled=move || authorizing.get() || pasting.get()
                    />
                </label>
                <button
                    class="btn btn-secondary mt-1"
                    type="submit"
                    disabled=move || authorizing.get() || pasting.get()
                >
                    <Show when=move || pasting.get()>
                        <span class="loading loading-spinner loading-xs"></span>
                    </Show>
                    "Sign in with token"
                </button>
            </form>
            {move || error.get().map(|message| view! {
                <div class="alert alert-soft alert-error mt-2 text-sm" role="alert">
                    <span>{message}</span>
                </div>
            })}
        </details>
    }
}
