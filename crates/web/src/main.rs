//! The Leptos client: a Mastodon web client whose discovery surface is the
//! recommendation Feed tab.
//!
//! This crate renders; it does not decide. All state lives in
//! `nodus-sdk` (the sibling `sdk` project), and every can happen is a
//! `Command` there. Scope, data layer and performance budget are specified in
//! `docs/` in this repository; the SDK is specified in `sdk/README.md`.

mod state;

use crate::state::bridge::{init, revision, with_sdk};

use leptos::prelude::*;
use nodus_sdk::StreamState;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}

/// Application shell.
#[component]
fn App() -> impl IntoView {
    init();
    let revision = revision();

    let signed_in = move || {
        revision.get();
        with_sdk(|sdk| sdk.state().session.get().is_some())
    };
    let stream_label = move || {
        revision.get();
        match with_sdk(|sdk| sdk.state().stream.get()) {
            StreamState::Connected => "connected",
            StreamState::Connecting => "connecting",
            StreamState::Reconnecting { .. } => "reconnecting",
            StreamState::Stopped => "stopped",
            StreamState::Disconnected => "disconnected",
        }
    };

    view! {
        <main class="shell">
            <h1>"nodus"</h1>
            <p>"Client shell — see docs/ for scope and plan, sdk/README.md for the state machine."</p>
            <p class="meta">
                "contract " {nodus_shared::CONTRACT_VERSION}
                " · session " {move || if signed_in() { "signed in" } else { "signed out" }}
                " · stream " {stream_label}
            </p>
        </main>
    }
}
