//! The application shell: the rail, the screen the address bar asks for, and the chrome
//! around both.
//!
//! Composition only: every screen comes from `crate::routes`, the chrome from `nodus-view`
//! or `crate::shell`.

use crate::router::{self, Route};
use crate::routes::callback::Callback;
use crate::routes::home::{FederatedTimeline, HomeTimeline, LocalTimeline};
use crate::routes::sign_in::SignIn;
use crate::shell;
use crate::state::bridge::{init, revision, with_sdk};

use leptos::prelude::*;
#[cfg(feature = "admin")]
use leptos::task::spawn_local;
use nodus_route::route::RouteView;
use nodus_ui::components::CardRenderer;
#[cfg(feature = "admin")]
use nodus_ui::icons::Icon;

/// Application shell: the rail, and the screen the address bar asks for.
#[component]
#[expect(
    clippy::too_many_lines,
    reason = "one arm per route: the match is the router, and splitting it hides a missing screen"
)]
pub(crate) fn App() -> impl IntoView {
    init();
    // Refreshes arrive on the SDK's own schedule; the views only render them.
    // Keeps the address bar and the back button in step with the rail.
    router::install();

    let revision = revision();
    let route = router::route();
    Effect::new(move |_| crate::state::runtime::remember_path(&route.get().path()));
    // A `Memo`, not a closure: the screen match below reads this, and a plain closure
    // would re-run it on every state notification — rebuilding the current screen
    // (disposing its owner, dropping its state, re-firing its effects) each time the
    // background tick or any SDK write bumped `revision`. A memo only notifies when the
    // *value* changes, which is exactly when the screen has to change too.
    let signed_in = Memo::new(move |_| {
        revision.get();
        with_sdk(|sdk| sdk.state().session.get().is_some())
    });

    let card: CardRenderer = std::sync::Arc::new(|status| {
        view! { <crate::routes::status::StatusCard status/> }.into_any()
    });

    view! {
        <nodus_ui::Frame
            signed_in=Signal::derive(move || signed_in.get())
            path=Signal::derive(move || router::route().get().path())
            extra=extra_doors()
            on_home=Callback::new(|()| router::navigate(Route::Home))
            inner=std::sync::Arc::new(move || {
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
                            Route::Feed if signed_in => {
                                let header: ChildrenFn = std::sync::Arc::new(|| {
                                    view! { <crate::routes::home::TimelineHeader/> }.into_any()
                                });
                                view! {
                                    <nodus_route::route::RouteView
                                        runtime=crate::state::runtime::runtime()
                                        route=Route::Feed
                                        card=card.clone()
                                        header=Some(header)
                                    />
                                }.into_any()
                            },
                            Route::Home if signed_in => view! { <HomeTimeline/> }.into_any(),
                            Route::Local if signed_in => view! { <LocalTimeline/> }.into_any(),
                            Route::Federated if signed_in => view! { <FederatedTimeline/> }.into_any(),
                            Route::Messages if signed_in => view! {
                                <RouteView
                                    runtime=crate::state::runtime::runtime()
                                    route=Route::Messages
                                    card=card.clone()
                                />
                            }
                            .into_any(),
                            Route::Notifications if signed_in => view! {
                                <RouteView
                                    runtime=crate::state::runtime::runtime()
                                    route=Route::Notifications
                                    card=card.clone()
                                />
                            }
                            .into_any(),
                            Route::Search if signed_in => view! {
                                <RouteView
                                    runtime=crate::state::runtime::runtime()
                                    route=Route::Search
                                    card=card.clone()
                                />
                            }
                            .into_any(),
                            Route::Directory if signed_in => view! {
                                <RouteView
                                    runtime=crate::state::runtime::runtime()
                                    route=Route::Directory
                                    card=card.clone()
                                />
                            }
                            .into_any(),
                            Route::Saved(shelf) if signed_in => view! {
                                <RouteView
                                    runtime=crate::state::runtime::runtime()
                                    route=Route::Saved(shelf)
                                    card=card.clone()
                                />
                            }
                            .into_any(),
                            Route::Scheduled if signed_in => view! {
                                <RouteView
                                    runtime=crate::state::runtime::runtime()
                                    route=Route::Scheduled
                                    card=card.clone()
                                />
                            }
                            .into_any(),
                            Route::Compose if signed_in => view! {
                                <RouteView
                                    runtime=crate::state::runtime::runtime()
                                    route=Route::Compose
                                    card=card.clone()
                                />
                            }
                            .into_any(),
                            Route::ComposeEdit(id) if signed_in => view! {
                                <RouteView
                                    runtime=crate::state::runtime::runtime()
                                    route=Route::ComposeEdit(id)
                                    card=card.clone()
                                />
                            }
                            .into_any(),
                            Route::Mutes if signed_in => view! {
                                <RouteView
                                    runtime=crate::state::runtime::runtime()
                                    route=Route::Mutes
                                    card=card.clone()
                                />
                            }
                            .into_any(),
                            Route::Blocks if signed_in => view! {
                                <RouteView
                                    runtime=crate::state::runtime::runtime()
                                    route=Route::Blocks
                                    card=card.clone()
                                />
                            }
                            .into_any(),
                            Route::Lists if signed_in => view! {
                                <RouteView
                                    runtime=crate::state::runtime::runtime()
                                    route=Route::Lists
                                    card=card.clone()
                                />
                            }
                            .into_any(),
                            Route::More if signed_in => view! {
                                <RouteView
                                    runtime=crate::state::runtime::runtime()
                                    route=Route::More
                                    card=card.clone()
                                />
                            }
                            .into_any(),
                            Route::Moderation(section) if signed_in => view! {
                                <RouteView
                                    runtime=crate::state::runtime::runtime()
                                    route=Route::Moderation(section)
                                    card=card.clone()
                                />
                            }
                            .into_any(),
                            Route::Admin(section) if signed_in => view! {
                                <RouteView
                                    runtime=crate::state::runtime::runtime()
                                    route=Route::Admin(section)
                                    card=card.clone()
                                />
                            }
                            .into_any(),
                            Route::Settings(section) if signed_in => view! {
                                <nodus_route::route::RouteView
                                    runtime=crate::state::runtime::runtime()
                                    route=Route::Settings(section)
                                    card=card.clone()
                                />
                            }.into_any(),
                            Route::Thread { id, .. } if signed_in => view! {
                                <RouteView
                                    runtime=crate::state::runtime::runtime()
                                    route=Route::Thread { host: String::new(), id }
                                    card=card.clone()
                                />
                            }
                            .into_any(),
                            Route::Account(id) if signed_in => view! {
                                <RouteView
                                    runtime=crate::state::runtime::runtime()
                                    route=Route::Account(id)
                                    card=card.clone()
                                />
                            }
                            .into_any(),
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
                            Route::More => {
                                view! { <SignIn note="Sign in to see every destination."/> }.into_any()
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
                                view! {
                                    <nodus_route::route::RouteView
                                        runtime=crate::state::runtime::runtime()
                                        route=Route::Thread { host, id }
                                        card=card.clone()
                                    />
                                }.into_any()
                            }
                            Route::Account(id) if id.contains('/') => view! {
                                <nodus_route::route::RouteView
                                    runtime=crate::state::runtime::runtime()
                                    route=Route::Account(id)
                                    card=card.clone()
                                />
                            }
                            .into_any(),
                            Route::Account(id) => match crate::state::runtime::runtime()
                                .public_host()
                            {
                                Some(host) => view! {
                                    <nodus_route::route::RouteView
                                        runtime=crate::state::runtime::runtime()
                                        route=Route::Account(format!("{host}/{id}"))
                                        card=card.clone()
                                    />
                                }
                                .into_any(),
                                None => {
                                    view! { <SignIn note="Sign in to read this."/> }.into_any()
                                }
                            }
                        }
            })/>
        <Show when=move || signed_in.get()>
            <shell::MobileBar/>
        </Show>
        <crate::media::ImageModal/>
        <div class="hidden xl:block">
            <nodus_ui::AppFooter host=Signal::derive(move || {
                with_sdk(|sdk| sdk.state().session.get().map(|session| session.host))
            })/>
        </div>
    }
}

/// The doors only a moderator build draws, as the shared frame wants them.
#[cfg(feature = "admin")]
#[expect(
    clippy::unnecessary_wraps,
    reason = "the lean build returns nothing, and one call site serves both"
)]
fn extra_doors() -> Option<nodus_ui::ExtraDoors> {
    Some(std::sync::Arc::new(|| staff_rail_items().into_any()))
}

/// The lean build has no moderation screens, so the rail has no extra doors.
#[cfg(not(feature = "admin"))]
fn extra_doors() -> Option<nodus_ui::ExtraDoors> {
    None
}

/// Whether two routes name the same rail section, even when they choose different subsections.
#[cfg(feature = "admin")]
fn same_section(current: &Route, target: &Route) -> bool {
    match (current, target) {
        (Route::Admin(_), Route::Admin(_)) | (Route::Moderation(_), Route::Moderation(_)) => true,
        _ => current == target,
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
    let target = route;
    let path = target.path();

    view! {
        <nodus_ui::RailEntry
            label=label
            hint=hint
            href=path
            icon=icon
            active=Signal::derive(move || same_section(&current.get(), &marker))
            on_click=Some(Callback::new(move |()| router::navigate(target.clone())))
        />
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

/// The probe behind the moderation doors.
#[cfg(feature = "admin")]
#[component]
fn StaffRail() -> impl IntoView {
    let standing = RwSignal::new(None);
    Effect::new(move |_| {
        revision().get();
        spawn_local(async move {
            let answer = crate::state::bridge::admin::staff().await;
            standing.set(Some(answer));
        });
    });

    view! {
        <Show when=move || !matches!(standing.get(), Some(crate::state::bridge::admin::Staff::No))>
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
