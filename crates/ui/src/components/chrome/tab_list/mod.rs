//! The strip of tabs a section opens with.
//!
//! The server built these by writing `<a role="tab" class="tab …">` into a string, and the
//! client built the same strip in two components, so a new section had to be added in three
//! places and the active marker disagreed on `/home` versus `/`. One component, taking the
//! tabs as data, keeps the strip and its active state in one definition.

use leptos::prelude::*;

/// One tab: where it goes and what it says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tab {
    /// The label the reader sees.
    pub label: String,
    /// The path the tab opens, absolute.
    pub href: String,
}

impl Tab {
    /// Builds a tab from anything string-like, so call sites can stay literal.
    pub fn new(label: impl Into<String>, href: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            href: href.into(),
        }
    }
}

/// The tab strip. `active` is the path the reader is on; a tab counts as selected when its
/// `href` is that path or a section the path lives under, so `/saved/favourites` marks its
/// own tab.
#[component]
pub fn TabList(
    tabs: Vec<Tab>,
    active: String,
    #[prop(optional)] on_navigate: Option<Callback<String>>,
) -> impl IntoView {
    view! {
        <div role="tablist" class="tabs tabs-box">
            <For each=move || tabs.clone() key=|tab| tab.href.clone() let:tab>
                {
                    let href = tab.href.clone();
                    let here = active.clone();
                    let is_active = href == here || here.starts_with(&format!("{href}/"));
                    let click_href = href.clone();
                    let navigate = on_navigate;
                    view! {
                        <a
                            role="tab"
                            class="tab"
                            class:tab-active=is_active
                            aria-selected=if is_active { "true" } else { "false" }
                            href=href
                            on:click=move |event| {
                                if let Some(navigate) = navigate {
                                    event.prevent_default();
                                    navigate.run(click_href.clone());
                                }
                            }
                        >
                            {tab.label}
                        </a>
                    }
                }
            </For>
        </div>
    }
}
