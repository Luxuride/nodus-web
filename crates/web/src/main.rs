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

use crate::router::Route;
use crate::routes::account::AccountView;
use crate::routes::callback::Callback;
use crate::routes::compose::{Compose, ComposeEdit};
use crate::routes::feed::Feed;
use crate::routes::home::Home;
use crate::routes::lists::Lists;
use crate::routes::notifications::Notifications;
use crate::routes::people::{Kind, PeopleList};
use crate::routes::saved::Saved;
use crate::routes::search::Search;
use crate::routes::settings::Settings;
use crate::routes::sign_in::SignIn;
use crate::routes::status::Thread;
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
                        <RailItem label="Feed" route=Route::Feed/>
                        <RailItem label="Home" route=Route::Home/>
                        <RailItem label="Notifications" route=Route::Notifications/>
                        <RailItem label="Search" route=Route::Search/>
                        <RailItem label="Saved" route=Route::Saved/>
                        <RailItem label="Lists" route=Route::Lists/>
                        <a
                            class="btn btn-primary btn-sm"
                            href=Route::Compose.path()
                            on:click=move |event| {
                                router::link(event, Route::Compose);
                            }
                        >
                            "Compose"
                        </a>
                        <RailItem label="Settings" route=Route::Settings/>
                    </ul>
                </nav>
                <main class="min-w-0 flex-1">
                    {move || {
                        // The callback comes first, before the signed-in gate: the
                        // instance redirects here while the app is still signed out, and
                        // this screen is what turns the code into a session.
                        //
                        // Everything the API serves needs a token, so a signed-out
                        // visitor gets the same answer on every gated route: what the
                        // click was for, or the rail reads as broken.
                        let signed_in = signed_in();
                        match route.get() {
                            Route::OauthCallback => view! { <Callback/> }.into_any(),
                            Route::Feed if signed_in => view! { <Feed/> }.into_any(),
                            Route::Home if signed_in => view! { <Home/> }.into_any(),
                            Route::Notifications if signed_in => {
                                view! { <Notifications/> }.into_any()
                            }
                            Route::Search if signed_in => view! { <Search/> }.into_any(),
                            Route::Saved if signed_in => view! { <Saved/> }.into_any(),
                            Route::Compose if signed_in => view! { <Compose/> }.into_any(),
                            Route::ComposeEdit(id) if signed_in => {
                                view! { <ComposeEdit id=id/> }.into_any()
                            }
                            Route::Mutes if signed_in => {
                                view! { <PeopleList id=String::new() kind=Kind::Mutes/> }
                                    .into_any()
                            }
                            Route::Blocks if signed_in => {
                                view! { <PeopleList id=String::new() kind=Kind::Blocks/> }
                                    .into_any()
                            }
                            Route::Lists if signed_in => view! { <Lists/> }.into_any(),
                            Route::Settings if signed_in => view! { <Settings/> }.into_any(),
                            Route::Thread(id) if signed_in => {
                                view! { <Thread id=id/> }.into_any()
                            }
                            Route::Account(id) if signed_in => {
                                view! { <AccountView id=id/> }.into_any()
                            }
                            Route::Feed => {
                                view! { <SignIn note="Sign in to see your feed."/> }.into_any()
                            }
                            Route::Home => {
                                view! { <SignIn note="Sign in to see your timeline."/> }.into_any()
                            }
                            Route::Notifications => {
                                view! {
                                    <SignIn note="Sign in to see your notifications."/>
                                }
                                    .into_any()
                            }
                            Route::Search => {
                                view! { <SignIn note="Sign in to search."/> }.into_any()
                            }
                            Route::Saved => {
                                view! { <SignIn note="Sign in to see what you saved."/> }
                                    .into_any()
                            }
                            Route::Lists => {
                                view! { <SignIn note="Sign in to manage your lists."/> }.into_any()
                            }
                            Route::Compose | Route::ComposeEdit(_) => {
                                view! { <SignIn note="Sign in to write a status."/> }.into_any()
                            }
                            Route::Mutes | Route::Blocks => {
                                view! { <SignIn note="Sign in to manage who you muted or blocked."/> }
                                    .into_any()
                            }
                            Route::Settings => {
                                view! {
                                    <SignIn note="Sign in to change your settings."/>
                                }
                                    .into_any()
                            }
                            Route::Thread(_) | Route::Account(_) => {
                                view! { <SignIn note="Sign in to read this."/> }.into_any()
                            }
                        }
                    }}
                </main>
            </div>
        </div>
    }
}

/// One entry in the rail: the class that marks the current screen, and a click that
/// keeps the address bar in step without a full page load.
///
/// `route` is owned and cloned twice because the active check and the click handler are
/// two closures; a route with an argument cannot be `Copy`, and leaking one to avoid the
/// clones would be a worse trade.
#[component]
fn RailItem(label: &'static str, route: Route) -> impl IntoView {
    let current = router::route();
    let marker = route.clone();
    let target = route.clone();
    let path = route.path();
    view! {
        <li>
            <a
                class="font-medium"
                class:menu-active=move || current.get() == marker
                href=path
                on:click=move |event| router::link(event, target.clone())
            >
                {label}
            </a>
        </li>
    }
}
