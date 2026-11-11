//! The phone shell's destinations, as data.
//!
//! The drawing is `nodus_ui::components::shell`; this file only says *which* destinations a
//! phone gets and where they point, so the addresses stay in the router and the markup stays in
//! the view crate. Two rules survive from when this was markup: five cells, because five fit a
//! 320 px screen, and the rest behind More because the rail does not fit under a thumb.

use leptos::prelude::*;
use nodus_ui::components::icons::Icon;
use nodus_ui::components::shell::BarTab;

use crate::router::{self, Route};

/// The five destinations in the bottom bar, in reading order.
fn bar() -> Vec<BarTab> {
    vec![
        BarTab::new("Home", Icon::Home, Route::Home.path()),
        BarTab::new("Messages", Icon::Mail, Route::Messages.path()),
        BarTab::new("Notifications", Icon::Bell, Route::Notifications.path()),
        BarTab::new("Search", Icon::Search, Route::Search.path()),
    ]
}

/// The bar above the bottom bar's content.
#[component]
pub fn MobileTop() -> impl IntoView {
    view! {
        <nodus_ui::components::shell::MobileTop
            home_path=Route::Home.path()
            on_home=Callback::new(|()| router::navigate(Route::Home))
        />
    }
}

/// Whether this route lives behind the More destination.
fn under_more(route: &Route) -> bool {
    matches!(
        route,
        Route::More
            | Route::Directory
            | Route::Saved(_)
            | Route::Scheduled
            | Route::Lists
            | Route::Compose
            | Route::ComposeEdit(_)
            | Route::Settings(_)
    )
}

/// The five destinations a phone gets under the thumb.
#[component]
pub fn MobileBar() -> impl IntoView {
    let current = router::route();
    view! {
        <nodus_ui::components::shell::MobileBar
            current=Signal::derive(move || current.get().path())
            tabs=bar()
            more=BarTab::new("More", Icon::More, Route::More.path())
            more_active=Signal::derive(move || under_more(&current.get()))
            on_navigate=Some(Callback::new(|path: String| {
                router::navigate(Route::from_path(&path));
            }))
        />
    }
}
