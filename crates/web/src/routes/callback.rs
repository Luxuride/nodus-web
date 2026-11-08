//! The instance's OAuth callback: finish the flow, then hand the token to the bridge.
//!
//! This screen exists while the app is still signed out — it is the only place that
//! can turn an authorization code into a session — so `main.rs` renders it before
//! the signed-in gate.

use leptos::prelude::*;

use crate::router::{Route, link, navigate};
use crate::state::alive::Alive;
use crate::state::bridge;
use crate::state::oauth;

/// The callback screen.
#[component]
pub fn Callback() -> impl IntoView {
    let error = RwSignal::new(None::<String>);

    // Read the query string before anything else: the exchange is async, and the
    // address bar is stripped right after so a reload cannot replay the code.
    let search = web_sys::window()
        .and_then(|window| window.location().search().ok())
        .unwrap_or_default();
    strip_query();

    // The exchange outlives this screen whenever the operator navigates away while it
    // runs, so the failure branch has to check before it writes.
    let alive = Alive::new();

    wasm_bindgen_futures::spawn_local(async move {
        match oauth::finish(&search).await {
            Ok(finished) if finished.purpose == oauth::Purpose::Admin => {
                match bridge::keep_admin_token(&finished.host, &finished.token).await {
                    // Back to the screen that asked, where the panels now answer.
                    Ok(()) => navigate(Route::Settings(crate::router::SECTION_DEFAULT.to_owned())),
                    Err(message) => alive.if_alive(|| error.set(Some(message))),
                }
            }
            Ok(finished) => {
                let (host, token) = (finished.host, finished.token);
                // `sign_in` verifies the token against the instance, installs the SDK
                // and persists the session itself; saving it again here would only
                // write the same two lines twice.
                match bridge::sign_in(&host, &token).await {
                    Ok(()) => navigate(Route::Feed),
                    Err(message) => alive.if_alive(|| error.set(Some(message))),
                }
            }
            Err(message) => alive.if_alive(|| error.set(Some(message))),
        }
    });

    view! {
        <section class="mx-auto w-full max-w-md px-4 py-10">
            <div class="card border border-base-300 bg-base-100 shadow-sm">
                <div class="card-body gap-4">
                    <h1 class="card-title">"Signing in"</h1>
                    {move || match error.get() {
                        None => view! {
                            <div class="flex items-center gap-2 text-sm opacity-70">
                                <span class="loading loading-spinner loading-xs"></span>
                                <span>"Finishing the handshake with your instance…"</span>
                            </div>
                        }
                            .into_any(),
                        Some(message) => view! {
                            <div class="alert alert-soft alert-error text-sm" role="alert">
                                <span>{message}</span>
                            </div>
                            // "/" is the sign-in screen while signed out, so this is the
                            // way back to a fresh attempt.
                            <a
                                class="btn btn-ghost btn-sm self-start"
                                href=Route::Feed.path()
                                on:click=move |event| link(event, Route::Feed)
                            >
                                "Back to sign-in"
                            </a>
                        }
                            .into_any(),
                    }}
                </div>
            </div>
        </section>
    }
}

/// Rewrites the address bar to the bare callback path, dropping `?code=…`.
///
/// The code is single-use and must not survive a reload, a bookmark or a shared URL.
/// `replace_state_with_url` rather than `push_state_with_url` so Back does not land
/// on it either.
fn strip_query() {
    if let Some(window) = web_sys::window() {
        if let Ok(history) = window.history() {
            let path = Route::OauthCallback.path();
            let _ = history.replace_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(&path));
        }
    }
}
