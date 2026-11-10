//! The phone's directory of every destination outside the bottom bar.

use leptos::prelude::*;
use nodus_web::shared::router::{Route, SECTION_DEFAULT, SHELF_DEFAULT};

use crate::components::icons::Icon;
use crate::components::shell::{BarTab, MorePage, SheetGroup};

fn groups() -> Vec<SheetGroup> {
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
            items: vec![BarTab::new("People", Icon::Users, Route::Directory.path())],
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

/// Every destination outside the bottom bar, with the current word kept visible.
#[component]
pub fn MoreScreen(
    path: String,
    #[prop(optional_no_strip)] on_navigate: Option<Callback<String>>,
) -> impl IntoView {
    let current = Signal::derive(move || path.clone());
    view! {
        <MorePage current=current groups=groups() on_navigate=on_navigate/>
    }
}
