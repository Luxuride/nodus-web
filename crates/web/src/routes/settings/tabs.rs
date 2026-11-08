//! The settings tab bar, and the list of sections it switches between.

use leptos::prelude::*;

use crate::router::Route;

/// One settings section at a time: cards stacked read as a wall, and the reader arrives
/// knowing which one they came for. Six sections, each answering one question — who am I,
/// what do I look like, what does Nodus do for me, what do I hide, where am I, and what may
/// I moderate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Account,
    Profile,
    Nodus,
    Filters,
    Instance,
}

impl Tab {
    /// Every section, in the order a reader is likely to want them.
    ///
    /// Every section. Moderation and administration are not settings: they are a separate
    /// area behind a staff check, reached from the rail, never from this bar.
    #[must_use]
    pub fn all() -> Vec<Self> {
        vec![
            Self::Account,
            Self::Profile,
            Self::Nodus,
            Self::Filters,
            Self::Instance,
        ]
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Account => "Account",
            Self::Profile => "Profile",
            Self::Nodus => "Nodus",
            Self::Filters => "Filters",
            Self::Instance => "Instance",
        }
    }

    /// What the section holds, for the tab's tooltip.
    /// The address this section lives at, so a tab is a link and not a stored state.
    pub fn slug(self) -> &'static str {
        match self {
            Self::Account => "account",
            Self::Profile => "profile",
            Self::Nodus => "nodus",
            Self::Filters => "filters",
            Self::Instance => "instance",
        }
    }

    /// The section a `/settings/{slug}` address names; an unknown slug reads as the
    /// first section rather than a blank screen.
    pub fn of(slug: &str) -> Self {
        Self::all()
            .into_iter()
            .find(|section| section.slug() == slug)
            .unwrap_or(Self::Account)
    }

    pub fn hint(self) -> &'static str {
        match self {
            Self::Account => "This session, connected accounts, muting and blocking",
            Self::Profile => "Display name, bio, clock and how sensitive media is shown",
            Self::Nodus => "Recommendations, background sync and which feeds the rail offers",
            Self::Filters => "Words and phrases to hide, and hashtags pinned to your profile",
            Self::Instance => "The instance's own identity and rules",
        }
    }
}

/// The tablist itself.
#[component]
pub fn SettingsTabs(tab: ReadSignal<Tab>) -> impl IntoView {
    view! {
        <div role="tablist" class="tabs tabs-box mb-4 w-fit flex-wrap">
            <For each=Tab::all key=|section| section.slug() let:section>
                <a
                    role="tab"
                    class="tab"
                    href=format!("/settings/{}", section.slug())
                    class:tab-active=move || tab.get() == section
                    title=section.hint()
                    on:click=move |event| crate::router::link(event, Route::Settings(section.slug().to_owned()))
                >
                    {section.label()}
                </a>
            </For>
        </div>
    }
}
