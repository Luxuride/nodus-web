//! The Home / Local / Federated switcher.

use leptos::prelude::*;

use super::Tab;

/// One tab per timeline, with the active one marked.
#[component]
pub fn TimelineTabs(tab: ReadSignal<Tab>, set_tab: WriteSignal<Tab>) -> impl IntoView {
    view! {
        <div role="tablist" class="tabs tabs-box w-fit">
            <For
                each=|| [Tab::Home, Tab::Local, Tab::Federated]
                key=|kind| format!("{kind:?}")
                let:kind
            >
                <button
                    role="tab"
                    class="tab gap-1.5"
                    class:tab-active=move || tab.get() == kind
                    title=kind.hint()
                    on:click=move |_| set_tab.set(kind)
                >
                    {kind.label()}
                </button>
            </For>
        </div>
    }
}
