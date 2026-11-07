//! The Leptos client: a Mastodon web client whose discovery surface is the
//! recommendation Feed tab.
//!
//! This crate renders; it does not decide. All state lives in
//! `nodus-sdk` (the sibling `sdk` project), and every can happen is a
//! `Command` there. Scope, data layer and performance budget are specified in
//! `docs/` in this repository; the SDK is specified in `sdk/README.md`.

mod router;
mod routes;
mod state;

use crate::routes::callback::Callback;
use crate::routes::feed::Feed;
use crate::routes::settings::Settings;
use crate::routes::sign_in::SignIn;
use crate::state::bridge::{init, revision, with_sdk};

use leptos::prelude::*;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
    // Started after the mount so the first render never waits on a network tick.
    crate::state::bridge::start_background_tick();
    // Impressions are written when a card leaves the screen, so the last screen's
    // worth of them is flushed on page hide (D36).
    crate::state::bridge::install_impression_flush();
}

/// Application shell: the rail, and the screen the address bar asks for.
#[component]
fn App() -> impl IntoView {
    init();
    // Refreshes arrive on the SDK's own schedule; the views only render them.
    // Keeps the address bar and the back button in step with the rail.
    router::install();

    let revision = revision();
    let route = router::route();
    let signed_in = move || {
        revision.get();
        with_sdk(|sdk| sdk.state().session.get().is_some())
    };

    view! {
        <div class="min-h-screen bg-base-200 text-base-content">
            <header class="border-b border-base-300 bg-base-100">
                <div class="mx-auto flex w-full max-w-5xl items-center gap-3 px-4 py-3">
                    <span class="text-lg font-semibold tracking-tight">"nodus"</span>
                    <span class="badge badge-ghost badge-sm">
                        "contract " {nodus_shared::CONTRACT_VERSION}
                    </span>
                </div>
            </header>
            // Two columns above `sm`: the rail is navigation, not decoration, so on a
            // narrow screen it becomes the header's only job (a single visible tab).
            <div class="mx-auto flex w-full max-w-5xl gap-4 px-2 py-4">
                <nav class="hidden w-44 shrink-0 sm:block">
                    <ul class="menu w-full gap-1 rounded-box bg-base-100 p-2 shadow-sm">
                        <li>
                            <a
                                class="font-medium"
                                class:menu-active=move || route.get() == router::Route::Feed
                                href=router::Route::Feed.path()
                                on:click=move |event| router::link(event, router::Route::Feed)
                            >
                                "Feed"
                            </a>
                        </li>
                        <li>
                            <a
                                class="font-medium"
                                class:menu-active=move || route.get() == router::Route::Settings
                                href=router::Route::Settings.path()
                                on:click=move |event| router::link(event, router::Route::Settings)
                            >
                                "Settings"
                            </a>
                        </li>
                    </ul>
                </nav>
                <main class="min-w-0 flex-1">
                    {move || {
                        // The callback comes first, before the signed-in gate: the
                        // instance redirects here while the app is still signed out, and
                        // this screen is what turns the code into a session.
                        match route.get() {
                            router::Route::OauthCallback => view! { <Callback/> }.into_any(),
                            router::Route::Feed if signed_in() => view! { <Feed/> }.into_any(),
                            router::Route::Settings if signed_in() => {
                                view! { <Settings/> }.into_any()
                            }
                            _ => view! { <SignIn/> }.into_any(),
                        }
                    }}
                </main>
            </div>
        </div>
    }
}
