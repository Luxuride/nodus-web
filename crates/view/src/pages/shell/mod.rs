//! The envelope a route gets when the server has the chrome but not the data.
//!
//! The heading, the section tabs and the skeleton are the same pieces the client mounts, so
//! the first paint reserves the space the real screen will use instead of jumping.

use leptos::prelude::*;

use crate::components::chrome::skeleton::Skeleton;
use crate::components::chrome::tab_list::Tab;
use crate::{PageHeader, TabList, TabListProps};

/// A heading, its section tabs, and a skeleton of the screen below them.
#[component]
pub fn ShellPage(
    #[prop(into)] title: String,
    #[prop(into)] description: String,
    tabs: Vec<Tab>,
    #[prop(into)] active: String,
    /// How many card frames the skeleton stands in for.
    skeleton_count: usize,
    #[prop(optional_no_strip)] on_navigate: Option<Callback<String>>,
) -> impl IntoView {
    let tabs = StoredValue::new(tabs);
    view! {
        <div class="mx-auto flex w-full flex-col gap-3 px-2 max-w-2xl">
            <PageHeader title=title description=description/>
            <Show when=move || tabs.with_value(|tabs| !tabs.is_empty())>
                {TabList(TabListProps {
                    tabs: tabs.get_value(),
                    active: active.clone(),
                    on_navigate,
                })}
            </Show>
            <Skeleton count=skeleton_count/>
        </div>
    }
}

/// The timeline's refusal, with the heading the client uses for Home.
#[component]
pub fn TimelineError(#[prop(into)] message: String) -> impl IntoView {
    view! {
        <div class="mx-auto flex w-full flex-col gap-3 px-2 max-w-2xl">
            <h1 class="text-xl font-semibold tracking-tight">"Home"</h1>
            <p class="alert alert-error mt-2 text-sm" role="alert">
                "Could not read the timeline: "
                {message}
            </p>
        </div>
    }
}
