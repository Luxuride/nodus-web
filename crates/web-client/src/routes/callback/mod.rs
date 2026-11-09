//! The instance's OAuth callback: finish the flow, then hand the token to the bridge.
//!
//! This screen exists while the app is still signed out — it is the only place that
//! can turn an authorization code into a session — so `main.rs` renders it before
//! the signed-in gate.

use leptos::prelude::*;

use crate::router::{Route, navigate};
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
        <nodus_view::Screen width=nodus_view::Width::Narrow>
            <nodus_view::pages::signing_in::SigningIn
                error=error.get()
                back=Some(Route::Feed.path())
                on_back=Callback::new(move |()| navigate(Route::Feed))
            />
        </nodus_view::Screen>
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
