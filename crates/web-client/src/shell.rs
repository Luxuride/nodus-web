//! The phone shell's destinations, as data.
//!
//! The drawing is `nodus_view::components::shell`; this file only says *which* destinations a
//! phone gets and where they point, so the addresses stay in the router and the markup stays in
//! the view crate. Two rules survive from when this was markup: five cells, because five fit a
//! 320 px screen, and the rest behind the sheet because the rail does not fit under a thumb.

use leptos::prelude::*;
use nodus_view::components::icons::Icon;
use nodus_view::components::shell::{BarTab, SheetGroup};

use crate::router::{self, Route, SECTION_DEFAULT, SHELF_DEFAULT};

/// The five destinations in the bottom bar, in reading order.
fn bar() -> Vec<BarTab> {
    vec![
        BarTab::new("Home", Icon::Home, Route::Home.path()),
        BarTab::new("Messages", Icon::Mail, Route::Messages.path()),
        BarTab::new("Notifications", Icon::Bell, Route::Notifications.path()),
        BarTab::new("Search", Icon::Search, Route::Search.path()),
    ]
}

/// Everything the bar does not hold, grouped the way the rail groups it.
fn sheet() -> Vec<SheetGroup> {
    vec![
        SheetGroup {
            title: "Read".to_owned(),
            items: vec![
                BarTab::new("Feed", Icon::Sparkle, Route::Feed.path()),
                BarTab::new("Home", Icon::Home, Route::Home.path()),
                BarTab::new("Local", Icon::Users, Route::Local.path()),
                BarTab::new("Federated", Icon::Globe, Route::Federated.path()),
            ],
        },
        SheetGroup {
            title: "Find".to_owned(),
            items: vec![BarTab::new(
                "People",
                Icon::Users,
                Route::Directory.path(),
            )],
        },
        SheetGroup {
            title: "Library".to_owned(),
            items: vec![
                BarTab::new(
                    "Saved",
                    Icon::Bookmark,
                    Route::Saved(SHELF_DEFAULT.to_owned()).path(),
                ),
                BarTab::new("Scheduled", Icon::Clock, Route::Scheduled.path()),
                BarTab::new("Lists", Icon::List, Route::Lists.path()),
            ],
        },
        SheetGroup {
            title: "Account".to_owned(),
            items: vec![
                BarTab::new("Compose", Icon::Pen, Route::Compose.path()),
                BarTab::new(
                    "Settings",
                    Icon::Settings,
                    Route::Settings(SECTION_DEFAULT.to_owned()).path(),
                ),
            ],
        },
    ]
}

/// The bar above the bottom bar's content.
#[component]
pub fn MobileTop() -> impl IntoView {
    let current = router::route();
    view! {
        <nodus_view::components::shell::MobileTop
            home_path=Route::Home.path()
            on_home=Callback::new(move |()| router::navigate(current.get_untracked().clone()))
        />
    }
}

/// The five destinations a phone gets under the thumb.
#[component]
pub fn MobileBar(open: RwSignal<bool>) -> impl IntoView {
    let current = router::route();
    view! {
        <nodus_view::components::shell::MobileBar
            open=open
            current=Signal::derive(move || current.get().path())
            tabs=bar()
            on_navigate=Some(Callback::new(|path: String| {
                router::navigate(Route::from_path(&path));
            }))
        />
    }
}

/// The rest of the destinations, in a sheet that slides up from the bottom bar.
#[component]
pub fn MobileSheet(open: RwSignal<bool>) -> impl IntoView {
    let current = router::route();
    view! {
        <nodus_view::components::shell::MobileSheet
            open=open
            current=Signal::derive(move || current.get().path())
            groups=sheet()
            on_navigate=Some(Callback::new(|path: String| {
                router::navigate(Route::from_path(&path));
            }))
        />
    }
}
