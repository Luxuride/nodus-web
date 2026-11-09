//! A strip of tabs that switch a list in place rather than opening an address.
//!
//! [`super::tab_list::TabList`] is the strip for sections, which are addresses: each tab is a
//! link that can be bookmarked, and the server paints it as one. The profile's three lists
//! are not addresses — they are one screen showing a different list — so their strip is
//! buttons. It is its own component rather than a mode of `TabList`, whose callers (the
//! document head and the settings and timeline strips) all pass links and would have to grow
//! a second shape for this one screen.

use leptos::prelude::*;

/// Buttons that switch what the screen below shows.
///
/// `labels` are both the key and the word on the button, so the caller's own state may name
/// its tabs and `on_select` hands that name back. `active` is a signal because the strip
/// stays mounted while the list under it changes.
#[component]
pub fn ButtonTabList(
    /// The tabs, in the order they are shown.
    labels: Vec<String>,
    /// The label of the tab on show.
    active: Signal<String>,
    /// Switches to the tab whose label this is. Absent where there is nothing to switch.
    #[prop(optional)]
    on_select: Option<Callback<String>>,
) -> impl IntoView {
    view! {
        <div role="tablist" class="tabs tabs-box w-fit">
            <For each=move || labels.clone() key=|label| label.clone() let:label>
                {
                    let on_show = label.clone();
                    let picked = label.clone();
                    view! {
                        <button
                            role="tab"
                            class="tab"
                            class:tab-active=move || active.get() == on_show
                            disabled=on_select.is_none()
                            on:click=move |_| {
                                if let Some(on_select) = on_select {
                                    on_select.run(picked.clone());
                                }
                            }
                        >
                            {label}
                        </button>
                    }
                }
            </For>
        </div>
    }
}
