//! One timeline, rendered by the shared route renderer.
//!
//! The screen owns the header and the banner slot; the store, the paging cursor and the
//! loading states belong to the runtime. Nothing about the list is written twice.

mod banner;

use leptos::prelude::*;

mod tabs;

mod header;
pub use header::TimelineHeader;
pub use tabs::PrimaryTabs;

pub use self::banner::Banner;


fn timeline(route: crate::router::Route) -> impl IntoView {
    let header: ChildrenFn = std::sync::Arc::new(|| view! { <TimelineHeader/> }.into_any());
    let banner: ChildrenFn = std::sync::Arc::new(|| view! { <Banner/> }.into_any());
    let card: nodus_ui::components::CardRenderer = std::sync::Arc::new(|status| {
        view! { <crate::routes::status::StatusCard status=status/> }.into_any()
    });
    view! {
        <nodus_route::route::RouteView
            runtime=crate::state::runtime::runtime()
            route=route
            card=card
            header=Some(header)
            banner=Some(banner)
        />
    }
}

/// The home timeline.
#[component]
pub fn HomeTimeline() -> impl IntoView {
    timeline(crate::router::Route::Home)
}

/// The local timeline.
#[component]
pub fn LocalTimeline() -> impl IntoView {
    timeline(crate::router::Route::Local)
}

/// The federated timeline.
#[component]
pub fn FederatedTimeline() -> impl IntoView {
    timeline(crate::router::Route::Federated)
}
