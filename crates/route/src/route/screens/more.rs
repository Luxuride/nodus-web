//! The phone's destinations that do not fit under the thumb.

use leptos::prelude::*;
use nodus_web::shared::router::{Route, SECTION_DEFAULT, SHELF_DEFAULT};

use crate::components::icons::Icon;
use crate::components::shell::{BarTab, MorePage, SheetGroup};

fn groups() -> Vec<SheetGroup> {
    vec![
        SheetGroup {
            title: "Browse".to_owned(),
            items: vec![
                BarTab::new("People", Icon::Users, Route::Directory.path()),
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

/// The destinations beyond the bar, with the section last opened kept visible.
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
