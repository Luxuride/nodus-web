//! One timeline, rendered by the shared route renderer.
//!
//! The screen owns the header and the banner slot; the store, the paging cursor and the
//! loading states belong to the runtime. Nothing about the list is written twice.

mod banner;

use leptos::prelude::*;
use nodus_sdk::Timeline;
use nodus_route::route::RouteView;

mod tabs;

mod header;
pub use header::TimelineHeader;
pub use tabs::PrimaryTabs;

use self::banner::Banner;
use crate::router::Route;

/// A timeline with its composer above it (home only) and a "Load more" foot.
#[component]
pub fn Home(which: Timeline) -> impl IntoView {
    let route = match which {
        Timeline::Local => Route::Local,
        Timeline::Federated => Route::Federated,
        _ => Route::Home,
    };

    let header: ChildrenFn = std::sync::Arc::new(|| view! { <TimelineHeader/> }.into_any());
    let banner: ChildrenFn = std::sync::Arc::new(|| view! { <Banner/> }.into_any());
    let card: nodus_ui::components::card_renderer::CardRenderer = std::sync::Arc::new(|status| {
        use leptos::prelude::IntoAny;
        view! { <crate::routes::status::StatusCard status=status/> }.into_any()
    });

    view! {
        <RouteView
            runtime=crate::state::runtime::runtime()
            route
            card
            header=Some(header)
            banner=Some(banner)
        />
    }
}
