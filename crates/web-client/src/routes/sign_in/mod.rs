//! Sign-in: authorize against the instance, with token pasting as the fallback.
//!
//! The primary path is the browser flow in [`crate::state::oauth`]: register the
//! client, send the browser to the instance, come back through `/oauth/callback`.
//! The user never types a credential here — the instance's own consent page is the
//! only place a password is entered, and this app never sees it.
//!
//! The form itself is the shared page; this module owns the two requests and the state
//! each one reports through.

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

    let authorize = move |()| {
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

    let submit_token = move |()| {
        if pasting.get_untracked() {
            return;
        }
        pasting.set(true);
        token_error.set(None);
        let host = host.get_untracked();
        let token = token.get_untracked();
        let alive = Alive::new();
        wasm_bindgen_futures::spawn_local(async move {
            if let Err(message) = sign_in(&host, &token).await {
                alive.if_alive(|| token_error.set(Some(message)));
            }
            alive.if_alive(|| pasting.set(false));
        });
    };

    view! {
        <nodus_pages::pages::sign_in::SignInPage
            note=note.map(str::to_owned)
            host=Signal::derive(move || host.get())
            token=Signal::derive(move || token.get())
            authorizing=Signal::derive(move || authorizing.get())
            pasting=Signal::derive(move || pasting.get())
            authorize_error=Signal::derive(move || authorize_error.get())
            token_error=Signal::derive(move || token_error.get())
            startup_error=Signal::derive(move || startup_error.get())
            on_host=Some(Callback::new(move |value: String| host.set(value)))
            on_token=Some(Callback::new(move |value: String| token.set(value)))
            on_authorize=Some(Callback::new(authorize))
            on_token_submit=Some(Callback::new(submit_token))
        />
    }
}
