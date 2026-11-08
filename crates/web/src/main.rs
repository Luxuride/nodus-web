//! The Leptos client: a Mastodon web client whose discovery surface is the
//! recommendation Feed tab.
//!
//! This crate renders; it does not decide. All state lives in
//! `nodus-sdk` (the sibling `sdk` project), and every can happen is a
//! `Command` there. Scope, data layer and performance budget are specified in
//! `docs/` in this repository; the SDK is specified in `sdk/README.md`.

mod media;
mod platform;
mod public;
mod router;
mod routes;
mod scroll;
mod shell;
mod state;
mod theme;
mod ui;

use crate::router::Route;
use crate::routes::account::AccountView;
use crate::routes::callback::Callback;
use crate::routes::compose::{Compose, ComposeEdit};
use crate::routes::conversations::Conversations;
use crate::routes::directory::Directory;
use crate::routes::feed::Feed;
use crate::routes::home::Home;
use crate::routes::lists::Lists;
use crate::routes::notifications::Notifications;
use crate::routes::people::{Kind, PeopleList};
use crate::routes::saved::Saved;
use crate::routes::scheduled::Scheduled;
use crate::routes::search::Search;
use crate::routes::settings::Settings;
use crate::routes::sign_in::SignIn;
use crate::routes::status::{PublicThread, Thread};
use crate::state::bridge::{init, revision, with_sdk};
use crate::ui::icons::Icon;

use leptos::prelude::*;
#[cfg(feature = "admin")]
use leptos::task::spawn_local;
use nodus_sdk::Timeline;

use wasm_bindgen::JsCast as _;

fn main() {
    console_error_panic_hook::set_once();
    // Before the first mount, so app-lifetime signals have no owner to be disposed
    // with (see `routes::settings::sync::install`).
    crate::routes::settings::sync::install();
    // Same reason: the revision signal is written by socket frames and ticks, which
    // outlive every screen, so it must not belong to one.
    crate::state::bridge::install_reactive_root();
    crate::state::clock_format::install();
    crate::state::feeds::install();
    // The expander's signal, created with no owner for the same reason.
    crate::media::install();
    // The operating system decides light or dark, and may change its mind while the tab
    // is open; `index.html` also sets the attribute before wasm arrives, so the first
    // paint is already correct.
    crate::theme::follow_system();
    // The server painted the first page into `#app`; the client replaces it, using the data
    // that page carried (`state::bridge::seed` read it a line earlier). Mounting *into* the
    // node instead of appending to `<body>` is what makes the takeover a replacement rather
    // than a second copy of the page underneath the first.
    crate::state::bridge::seed::install();
    mount_app();
    // Started after the mount so the first render never waits on a network tick.
    crate::state::bridge::start_background_tick();
    // Impressions are written when a card leaves the screen, so the last screen's
    // worth of them is flushed on page hide (D36).
    crate::state::bridge::install_impression_flush();
}

/// Mounts the app into the server-painted node, or into `<body>` when there is none.
///
/// A static deployment (no SSR host) has no `#app`, and `mount_to_body` is the right answer
/// there; the SSR document always has one.
fn mount_app() {
    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return;
    };
    let Some(host) = document
        .get_element_by_id("app")
        .and_then(|node| node.dyn_into::<web_sys::HtmlElement>().ok())
    else {
        leptos::mount::mount_to_body(App);
        return;
    };
    // Cleared only now, after the island has been read: the island lives inside this node.
    host.set_inner_html("");
    // `mount_to` hands back a handle that unmounts on drop; this view lives for the tab.
    leptos::mount::mount_to(host, App).forget();
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
    // A `Memo`, not a closure: the screen match below reads this, and a plain closure
    // would re-run it on every state notification — rebuilding the current screen
    // (disposing its owner, dropping its state, re-firing its effects) each time the
    // background tick or any SDK write bumped `revision`. A memo only notifies when the
    // *value* changes, which is exactly when the screen has to change too.
    let signed_in = Memo::new(move |_| {
        revision.get();
        with_sdk(|sdk| sdk.state().session.get().is_some())
    });

    // The sheet is a signal, not a route: opening it must not change the address bar.
    let sheet = RwSignal::new(false);

    view! {
        <div class="min-h-screen bg-base-200 text-base-content">
            <Show when=move || signed_in.get()>
                <shell::MobileTop/>
            </Show>
            // No top bar: the rail already carries the name and the navigation, and a
            // bar above it only cost vertical space on every screen.
            // Two columns above `sm`: the rail is navigation, not decoration, so on a
            // narrow screen it becomes the header's only job (a single visible tab).
            // Desktop: rail, composer, content. Narrower: composer above the rail,
            // rail above the content — writing is still one press away, and nothing
            // pushes the timeline off-screen.
            <div class="mx-auto flex w-full max-w-6xl flex-col gap-4 px-2 py-4 xl:flex-row">
                <Show when=move || signed_in.get()>
                <nav class="hidden shrink-0 xl:block xl:w-44">
                    <ul class="menu w-full flex-row flex-wrap gap-1 rounded-box bg-base-100 p-2 shadow-sm xl:flex-col xl:flex-nowrap">
                        {nodus_view::rail()
                            .into_iter()
                            .map(|group| {
                                view! {
                                    <li class="menu-title hidden xl:block">{group.title}</li>
                                    {group
                                        .destinations
                                        .iter()
                                        .map(|destination| {
                                            let href = destination.href;
                                            let active = move || router::route().get().path() == href;
                                            view! {
                                                <li>
                                                    <a
                                                        class="flex w-full items-center gap-2 font-medium"
                                                        class:menu-active=active
                                                        href=href
                                                        title=destination.hint
                                                    >
                                                        <Icon icon=icon_of(destination.label)/>
                                                        <span class="truncate">{destination.label}</span>
                                                    </a>
                                                </li>
                                            }
                                        })
                                        .collect_view()}
                                }
                            })
                            .collect_view()}
                        {staff_rail_items()}
                    </ul>
                </nav>
                </Show>
                <main class="min-w-0 flex-1 pb-[calc(5.5rem+env(safe-area-inset-bottom))] xl:pb-0">
                    {move || {
                        // The callback comes first, before the signed-in gate: the
                        // instance redirects here while the app is still signed out, and
                        // this screen is what turns the code into a session.
                        //
                        // Everything the API serves needs a token, so a signed-out
                        // visitor gets the same answer on every gated route: what the
                        // click was for, or the rail reads as broken.
                        let signed_in = signed_in.get();
                        match route.get() {
                            Route::OauthCallback => view! { <Callback/> }.into_any(),
                            Route::Feed if signed_in => view! { <Feed/> }.into_any(),
                            Route::Home if signed_in => {
                                view! { <Home which=Timeline::Home/> }.into_any()
                            }
                            Route::Local if signed_in => {
                                view! { <Home which=Timeline::Local/> }.into_any()
                            }
                            Route::Federated if signed_in => {
                                view! { <Home which=Timeline::Federated/> }.into_any()
                            }
                            Route::Messages if signed_in => {
                                view! { <Conversations/> }.into_any()
                            }
                            Route::Notifications if signed_in => {
                                view! { <Notifications/> }.into_any()
                            }
                            Route::Search if signed_in => view! { <Search/> }.into_any(),
                            Route::Directory if signed_in => view! { <Directory/> }.into_any(),
                            Route::Saved(shelf) if signed_in => view! { <Saved shelf=shelf/> }.into_any(),
                            Route::Scheduled if signed_in => view! { <Scheduled/> }.into_any(),
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
                            Route::Moderation(section) if signed_in => {
                                #[cfg(feature = "admin")]
                                {
                                    view! { <crate::routes::moderation::Moderation section=section/> }
                                        .into_any()
                                }
                                #[cfg(not(feature = "admin"))]
                                {
                                    let _ = &section;
                                    view! {
                                        <p class="text-sm opacity-70">
                                            "This build has no moderation screens."
                                        </p>
                                    }
                                    .into_any()
                                }
                            }
                            Route::Admin(section) if signed_in => {
                                #[cfg(feature = "admin")]
                                {
                                    view! { <crate::routes::admin::Admin section=section/> }.into_any()
                                }
                                #[cfg(not(feature = "admin"))]
                                {
                                    let _ = &section;
                                    view! {
                                        <p class="text-sm opacity-70">
                                            "This build has no moderation screens. A moderator build "
                                            "is " <code>"trunk build --release --features admin"</code>
                                            "."
                                        </p>
                                    }
                                    .into_any()
                                }
                            }
                            Route::Settings(section) if signed_in => view! { <Settings section=section/> }.into_any(),
                            Route::Thread { id, .. } if signed_in => {
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
                            Route::Local => {
                                view! { <SignIn note="Sign in to see this instance's posts."/> }
                                    .into_any()
                            }
                            Route::Federated => {
                                view! { <SignIn note="Sign in to see what this instance received."/> }
                                    .into_any()
                            }
                            Route::Messages => {
                                view! { <SignIn note="Sign in to read your messages."/> }
                                    .into_any()
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
                            Route::Directory => {
                                view! { <SignIn note="Sign in to find people."/> }.into_any()
                            }
                            Route::Saved(_) => {
                                view! { <SignIn note="Sign in to see what you saved."/> }
                                    .into_any()
                            }
                            Route::Scheduled => {
                                view! { <SignIn note="Sign in to schedule a post."/> }.into_any()
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
                            Route::Settings(_) => {
                                view! {
                                    <SignIn note="Sign in to change your settings."/>
                                }
                                    .into_any()
                            }
                            Route::Admin(_) | Route::Moderation(_) => {
                                view! { <SignIn note="Sign in to moderate."/> }.into_any()
                            }
                            // A shared link works without a session: the address names the
                            // host, so the post is read from there anonymously. A legacy
                            // `/status/{id}` link falls back to the instance this browser
                            // already used, or asks which one.
                            Route::Thread { host, id } => {
                                let host = if host.is_empty() {
                                    crate::state::bridge::session::remembered_host()
                                        .unwrap_or_default()
                                } else {
                                    host
                                };
                                view! { <PublicThread host=host id=id/> }.into_any()
                            }
                            Route::Account(_) => {
                                view! { <SignIn note="Sign in to read this."/> }.into_any()
                            }
                        }
                    }}
                </main>
            </div>
            <Show when=move || signed_in.get()>
                <shell::MobileBar open=sheet/>
                <shell::MobileSheet open=sheet/>
            </Show>
            <crate::media::ImageModal/>
            <Footer/>
        </div>
    }
}

/// The foot of every screen: what this is, and the two links a client owes its server.
#[component]
fn Footer() -> impl IntoView {
    let host = with_sdk(|sdk| sdk.state().session.get().map(|session| session.host));
    view! {
        <footer class="mt-8 border-t border-base-300 px-4 py-6 text-xs opacity-70">
            <div class="mx-auto flex w-full max-w-5xl flex-wrap items-center gap-x-3 gap-y-2">
                <span class="font-semibold tracking-tight text-base-content">"nodus"</span>
                <span>"A Mastodon client with a recommendation feed."</span>
                {move || host.clone().map(|host| view! {
                    <a
                        class="link link-hover"
                        href=format!("https://{host}/about")
                        target="_blank"
                        rel="noreferrer noopener"
                    >
                        "About this instance"
                    </a>
                })}
                <a
                    class="link link-hover"
                    href="https://github.com/nodus-social"
                    target="_blank"
                    rel="noreferrer noopener"
                >
                    "Source"
                </a>
                <span class="ml-auto opacity-60">
                    "No ads, no tracking, no algorithm you cannot see."
                </span>
            </div>
        </footer>
    }
}

/// One entry in the rail: the class that marks the current screen, and a click that
/// keeps the address bar in step without a full page load.
///
/// `route` is owned and cloned twice because the active check and the click handler are
/// two closures; a route with an argument cannot be `Copy`, and leaking one to avoid the
/// clones would be a worse trade.
#[cfg(feature = "admin")]
#[component]
fn RailItem(label: &'static str, icon: Icon, hint: &'static str, route: Route) -> impl IntoView {
    let current = router::route();
    let marker = route.clone();
    let target = route.clone();
    let path = route.path();
    view! {
        <li>
            <a
                // Full width, so the whole row is the hit area: a rail whose padding does
                // nothing reads as a rail whose buttons are broken.
                class="flex w-full items-center gap-2 font-medium"
                // The hint is what a label cannot say: "Saved" does not tell you it holds
                // both favourites and bookmarks.
                title=hint
                class:menu-active=move || current.get() == marker
                href=path
                on:click=move |event| router::link(event, target.clone())
            >
                <Icon icon=icon/>
                <span class="truncate">{label}</span>
            </a>
        </li>
    }
}

/// The moderation doors, in a build that carries the screens.
///
/// Shown only once the instance has answered that this account may moderate: a rail entry
/// that leads to "you are not a moderator" is a dead end, and `admin::staff()` is the only
/// honest source for that answer. The check re-runs when the session or the grant changes,
/// because both can arrive after the first paint.
#[cfg(feature = "admin")]
fn staff_rail_items() -> impl IntoView {
    view! { <StaffRail/> }
}

#[cfg(not(feature = "admin"))]
fn staff_rail_items() -> impl IntoView {}

/// The probe behind the moderation doors.
#[cfg(feature = "admin")]
#[component]
fn StaffRail() -> impl IntoView {
    let staff = RwSignal::new(false);
    Effect::new(move |_| {
        revision().get();
        spawn_local(async move {
            let answer = crate::state::bridge::admin::staff().await;
            staff.set(matches!(answer, crate::state::bridge::admin::Staff::Yes));
        });
    });

    view! {
        <Show when=move || staff.get()>
            <RailItem
                label="Moderation"
                icon=Icon::Shield
                hint="Reports, accounts, registrations and blocked domains"
                route=Route::Moderation(crate::router::MODERATION_DEFAULT.to_owned())
            />
            <RailItem
                label="Administration"
                icon=Icon::Globe
                hint="The dashboard, allowed domains and announcements"
                route=Route::Admin(crate::router::ADMIN_DEFAULT.to_owned())
            />
        </Show>
    }
}

/// The icon for a rail destination, chosen by the label `nodus_view::rail()` publishes.
///
/// The list of destinations and their order is not this function's business: it is the same
/// list the server draws, so a new screen appears in both paints or in neither.
fn icon_of(label: &str) -> Icon {
    match label {
        "Home" => Icon::Home,
        "Messages" => Icon::Mail,
        "Notifications" => Icon::Bell,
        "Search" => Icon::Search,
        "People" => Icon::Users,
        "Saved" => Icon::Bookmark,
        "Scheduled" => Icon::Clock,
        "Lists" => Icon::List,
        "Admin" => Icon::Shield,
        "Moderation" => Icon::Alert,
        _ => Icon::Settings,
    }
}
