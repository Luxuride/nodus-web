//! The profile tab strip.

use leptos::prelude::*;

use super::Tab;

/// Posts, followers, following.
#[component]
pub fn AccountTabs(tab: ReadSignal<Tab>, set_tab: WriteSignal<Tab>) -> impl IntoView {
    view! {
        <div role="tablist" class="tabs tabs-box w-fit">
            <For each=|| [Tab::Posts, Tab::Followers, Tab::Following] key=|which| which.label() let:which>
                <button
                    role="tab"
                    class="tab"
                    class:tab-active=move || tab.get() == which
                    on:click=move |_| set_tab.set(which)
                >
                    {which.label()}
                </button>
            </For>
        </div>
    }
}
