use leptos::prelude::*;

use crate::router::{self, Route};
use crate::state::feeds;

/// The timelines this reader can read, under the composer.
///
/// These are links, not a tab signal: `/home`, `/feed`, `/local` and `/federated`
/// are addresses, so a reload keeps the timeline the reader was reading and a
/// shared link lands on the same one.
#[component]
pub fn PrimaryTabs() -> impl IntoView {
    let route = router::route();
    let changed = feeds::revision();

    let items = move || {
        if let Some(changed) = changed {
            changed.get();
        }
        let mut items: Vec<(Route, &'static str)> = vec![
            (Route::Home, "Home"),
            (Route::Feed, "Feed"),
        ];
        if feeds::enabled(feeds::Feed::Local) {
            items.push((Route::Local, "Local"));
        }
        if feeds::enabled(feeds::Feed::Federated) {
            items.push((Route::Federated, "Federated"));
        }
        items
    };

    view! {
        <div role="tablist" class="tabs tabs-box mb-2 w-full flex-wrap">
            <For each=items key=|(target, _)| target.path() let:item>
                {
                    let (target, label) = item;
                    let active = target.clone();
                    let click = target.clone();
                    view! {
                        <a
                            role="tab"
                            class="tab"
                            href=target.path()
                            class:tab-active=move || route.get() == active
                            on:click=move |event| router::link(event, click.clone())
                        >
                            {label}
                        </a>
                    }
                }
            </For>
        </div>
    }
}
