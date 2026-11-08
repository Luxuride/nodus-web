//! The shell a phone gets: a bar at the bottom, a sheet for everything else.
//!
//! The rail is the right shape for a wide window — twelve destinations, always in the same
//! place, each one visible. On a phone the same list eats a third of the screen before the
//! first post, so the narrow layout keeps the five destinations a phone is used to having
//! under the thumb and puts the rest behind one button. Nothing becomes unreachable: the
//! sheet is the rail, in a drawer.

use leptos::prelude::*;

use crate::router::{self, Route};
use crate::ui::icons::Icon;

/// The bar above the bottom bar's content: who this is, and the one action worth a
/// permanent button.
#[component]
pub fn MobileTop() -> impl IntoView {
    let compose = Route::Compose.path();
    view! {
        <header class="sticky top-0 z-30 border-b border-base-300 bg-base-200/95 backdrop-blur xl:hidden">
            <div class="mx-auto flex max-w-2xl items-center gap-2 px-3 py-2">
                <a class="font-semibold tracking-tight" href=Route::Home.path()>
                    "nodus"
                </a>
                <a
                    class="btn btn-primary btn-sm ml-auto gap-1"
                    href=compose
                    on:click=move |event| router::link(event, Route::Compose)
                >
                    <Icon icon=Icon::Pen class="size-4"/>
                    <span>"Write"</span>
                </a>
            </div>
        </header>
    }
}

/// The five destinations a phone gets under the thumb, in reading order.
#[component]
pub fn MobileBar(open: RwSignal<bool>) -> impl IntoView {
    let current = router::route();
    view! {
        <nav
            class="fixed inset-x-0 bottom-0 z-40 border-t border-base-300 bg-base-100/95 backdrop-blur xl:hidden"
            style="padding-bottom: env(safe-area-inset-bottom)"
            aria-label="Primary"
        >
            <ul class="mx-auto flex max-w-lg items-stretch px-1">
                <Tab label="Home" icon=Icon::Home route=Route::Home current=current/>
                <Tab label="Messages" icon=Icon::Mail route=Route::Messages current=current/>
                <Tab
                    label="Notifications"
                    icon=Icon::Bell
                    route=Route::Notifications
                    current=current
                />
                <Tab label="Search" icon=Icon::Search route=Route::Search current=current/>
                <li class="flex-1">
                    <button
                        type="button"
                        class="flex w-full flex-col items-center gap-0.5 py-2 text-[0.65rem] font-medium leading-none"
                        on:click=move |_| open.update(|shown| *shown = !*shown)
                        aria-label="All destinations"
                    >
                        <Icon icon=Icon::More class="size-5"/>
                        <span class="max-w-full truncate">"More"</span>
                    </button>
                </li>
            </ul>
        </nav>
    }
}

/// One destination in the bottom bar: icon above the label, and the whole cell is the hit
/// area. The label is small because five of them have to fit on a 320 px screen, and it is
/// truncated rather than wrapped because a two-line bar would change height with the route.
#[component]
fn Tab(label: &'static str, icon: Icon, route: Route, current: RwSignal<Route>) -> impl IntoView {
    let marker = route.clone();
    let faded = marker.clone();
    let target = route.clone();
    let path = route.path();
    view! {
        <li class="flex-1">
            <a
                class="flex flex-col items-center gap-0.5 py-2 text-[0.65rem] font-medium leading-none"
                class:opacity-60=move || current.get() != faded
                class:text-primary=move || current.get() == marker
                href=path
                on:click=move |event| router::link(event, target.clone())
                title=label
            >
                <Icon icon=icon class="size-5"/>
                <span class="max-w-full truncate">{label}</span>
            </a>
        </li>
    }
}

/// The rest of the destinations, in a sheet that slides up from the bottom bar.
///
/// A sheet rather than a page: opening it must not change the address bar, so closing it
/// returns the reader to exactly the screen they were on.
#[component]
pub fn MobileSheet(open: RwSignal<bool>) -> impl IntoView {
    let current = router::route();
    view! {
        <div
            class="fixed inset-0 z-50 xl:hidden"
            class:hidden=move || !open.get()
            role="dialog"
            aria-modal="true"
            aria-label="All destinations"
        >
            <button
                type="button"
                class="absolute inset-0 w-full bg-black/40"
                aria-label="Close"
                on:click=move |_| open.set(false)
            ></button>
            <div class="absolute inset-x-0 bottom-0 max-h-[80vh] overflow-y-auto rounded-t-2xl border-t border-base-300 bg-base-100 pb-24 shadow-2xl">
                <div class="flex items-center justify-between px-4 pt-3">
                    <h2 class="text-sm font-semibold">"Everything else"</h2>
                    <button
                        type="button"
                        class="btn btn-ghost btn-sm btn-square"
                        aria-label="Close"
                        on:click=move |_| open.set(false)
                    >
                        <Icon icon=Icon::Close/>
                    </button>
                </div>
                <ul class="menu w-full px-2 py-2">
                    <li class="menu-title">"Read"</li>
                    <SheetItem label="Feed" icon=Icon::Sparkle route=Route::Feed current=current open=open/>
                    <SheetItem label="Home" icon=Icon::Home route=Route::Home current=current open=open/>
                    <SheetItem label="Local" icon=Icon::Users route=Route::Local current=current open=open/>
                    <SheetItem
                        label="Federated"
                        icon=Icon::Globe
                        route=Route::Federated
                        current=current
                        open=open
                    />
                    <li class="menu-title">"Find"</li>
                    <SheetItem label="People" icon=Icon::Users route=Route::Directory current=current open=open/>
                    <li class="menu-title">"Library"</li>
                    <SheetItem label="Saved" icon=Icon::Bookmark route=Route::Saved(crate::router::SHELF_DEFAULT.to_owned()) current=current open=open/>
                    <SheetItem label="Scheduled" icon=Icon::Clock route=Route::Scheduled current=current open=open/>
                    <SheetItem label="Lists" icon=Icon::List route=Route::Lists current=current open=open/>
                    <li class="menu-title">"Account"</li>
                    <SheetItem label="Compose" icon=Icon::Pen route=Route::Compose current=current open=open/>
                    <SheetItem label="Settings" icon=Icon::Settings route=Route::Settings(crate::router::SECTION_DEFAULT.to_owned()) current=current open=open/>
                </ul>
            </div>
        </div>
    }
}

/// One row in the sheet: closes the sheet, then navigates, so the reader sees the screen
/// rather than the drawer sliding away over it.
#[component]
fn SheetItem(
    label: &'static str,
    icon: Icon,
    route: Route,
    current: RwSignal<Route>,
    open: RwSignal<bool>,
) -> impl IntoView {
    let marker = route.clone();
    let target = route.clone();
    let path = route.path();
    view! {
        <li>
            <a
                class="flex w-full items-center gap-2 py-2 font-medium"
                class:menu-active=move || current.get() == marker
                href=path
                on:click=move |event| {
                    open.set(false);
                    router::link(event, target.clone());
                }
            >
                <Icon icon=icon/>
                <span>{label}</span>
            </a>
        </li>
    }
}
