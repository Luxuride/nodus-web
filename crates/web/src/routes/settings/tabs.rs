//! The settings tab bar, and the list of sections it switches between.

use leptos::prelude::*;

/// One settings section at a time: seven cards stacked read as a wall, and the reader
/// arrives knowing which one they came for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Account,
    Profile,
    Privacy,
    Filters,
    Tags,
    Instance,
    Administration,
}

impl Tab {
    /// Every section, in the order a reader is likely to want them.
    pub const ALL: [Self; 7] = [
        Self::Account,
        Self::Profile,
        Self::Privacy,
        Self::Filters,
        Self::Tags,
        Self::Instance,
        Self::Administration,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Account => "Account",
            Self::Profile => "Profile",
            Self::Privacy => "Privacy",
            Self::Filters => "Filters",
            Self::Tags => "Tags",
            Self::Instance => "Instance",
            Self::Administration => "Administration",
        }
    }

    /// What the section holds, for the tab's tooltip.
    pub fn hint(self) -> &'static str {
        match self {
            Self::Account => "This session, connected accounts, muting and blocking",
            Self::Profile => "Display name, bio, and how sensitive media is shown",
            Self::Privacy => "Background sync consent and the token it uses",
            Self::Filters => "Words and phrases to hide or warn about",
            Self::Tags => "Hashtags pinned to your profile",
            Self::Instance => "The rules this instance publishes",
            Self::Administration => "Invites, pending approvals and the moderation screens",
        }
    }
}

/// The tablist itself.
#[component]
pub fn SettingsTabs(tab: ReadSignal<Tab>, set_tab: WriteSignal<Tab>) -> impl IntoView {
    view! {
        <div role="tablist" class="tabs tabs-box mb-4 w-fit flex-wrap">
            <For each=|| Tab::ALL key=|section| format!("{section:?}") let:section>
                <button
                    role="tab"
                    class="tab"
                    class:tab-active=move || tab.get() == section
                    title=section.hint()
                    on:click=move |_| set_tab.set(section)
                >
                    {section.label()}
                </button>
            </For>
        </div>
    }
}
