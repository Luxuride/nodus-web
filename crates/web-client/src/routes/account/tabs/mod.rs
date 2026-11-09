//! The profile tab strip.
//!
//! The strip is markup and lives in `nodus-view` (`ButtonTabList`: buttons, because the
//! three lists are one screen showing a different list, not three addresses). What stays
//! here is which tab is on show — a `Tab`, which is the client's own word for the list
//! under the strip.

use leptos::prelude::*;
use nodus_view::chrome::ButtonTabList;

use super::Tab;

/// Posts, followers, following.
#[component]
pub fn AccountTabs(tab: ReadSignal<Tab>, set_tab: WriteSignal<Tab>) -> impl IntoView {
    let labels = Tab::ALL
        .iter()
        .map(|which| which.label().to_owned())
        .collect::<Vec<_>>();

    view! {
        <ButtonTabList
            labels=labels
            active=Signal::derive(move || tab.get().label().to_owned())
            on_select=Callback::new(move |label: String| set_tab.set(Tab::of(&label)))
        />
    }
}
