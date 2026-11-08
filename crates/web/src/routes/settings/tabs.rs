//! The settings tab bar, and the list of sections it switches between.

use leptos::prelude::*;

use crate::router::Route;

/// One settings section at a time: seven cards stacked read as a wall, and the reader
/// arrives knowing which one they came for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Account,
    Profile,
    Privacy,
    Feeds,
    Filters,
    Tags,
    Instance,
    Administration,
}

impl Tab {
    /// Every section, in the order a reader is likely to want them.
    ///
    /// Administration is listed only in a moderator build: a tab that answers "this build
    /// has no moderation screens" is a dead end the reader has to walk into to discover.
    #[must_use]
    pub fn all() -> Vec<Self> {
        let mut sections = vec![
            Self::Account,
            Self::Profile,
            Self::Privacy,
            Self::Feeds,
            Self::Filters,
            Self::Tags,
            Self::Instance,
        ];
        if cfg!(feature = "admin") {
            sections.push(Self::Administration);
        }
        sections
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Account => "Account",
            Self::Profile => "Profile",
            Self::Privacy => "Privacy",
            Self::Feeds => "Feeds",
            Self::Filters => "Filters",
            Self::Tags => "Tags",
            Self::Instance => "Instance",
            Self::Administration => "Administration",
        }
    }

    /// What the section holds, for the tab's tooltip.
    /// The address this section lives at, so a tab is a link and not a stored state.
    pub fn slug(self) -> &'static str {
        match self {
            Self::Account => "account",
            Self::Profile => "profile",
            Self::Privacy => "privacy",
            Self::Filters => "filters",
            Self::Tags => "tags",
            Self::Feeds => "feeds",
            Self::Instance => "instance",
            Self::Administration => "administration",
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
            Self::Profile => "Display name, bio, and how sensitive media is shown",
            Self::Privacy => "Background sync consent and the token it uses",
            Self::Feeds => "Which timelines the navigation offers",
            Self::Filters => "Words and phrases to hide or warn about",
            Self::Tags => "Hashtags pinned to your profile",
            Self::Instance => "The rules this instance publishes",
            Self::Administration => "Invites, pending approvals and the moderation screens",
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
