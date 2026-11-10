//! The settings sections and the strip that switches between them.

use leptos::prelude::*;

/// One section of Settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsSection {
    Account,
    Profile,
    Nodus,
    Filters,
    Instance,
}

impl SettingsSection {
    pub const ALL: [Self; 5] = [
        Self::Account,
        Self::Profile,
        Self::Nodus,
        Self::Filters,
        Self::Instance,
    ];

    pub fn slug(self) -> &'static str {
        match self {
            Self::Account => "account",
            Self::Profile => "profile",
            Self::Nodus => "nodus",
            Self::Filters => "filters",
            Self::Instance => "instance",
        }
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

    pub fn hint(self) -> &'static str {
        match self {
            Self::Account => "Who you are signed in as, and the other accounts this browser holds.",
            Self::Profile => "Your display name, bio, clock and media choices.",
            Self::Nodus => "Timelines, background sync and featured tags.",
            Self::Filters => "Phrases that hide a post or put it behind a warning.",
            Self::Instance => "What this server says about itself.",
        }
    }

    pub fn of(slug: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|section| section.slug() == slug)
            .unwrap_or(Self::Account)
    }
}

/// The tab strip at the top of Settings.
#[component]
pub fn SettingsTabs(
    #[prop(into)] section: Signal<SettingsSection>,
    #[prop(optional_no_strip)] on_navigate: Option<Callback<SettingsSection>>,
) -> impl IntoView {
    view! {
        <div role="tablist" class="tabs tabs-box mb-4 w-full flex-wrap sm:w-fit">
            <For each=|| SettingsSection::ALL key=|item| item.slug() let:item>
                {
                    let href = format!("/settings/{}", item.slug());
                    view! {
                        <a
                            role="tab"
                            class="tab"
                            class:tab-active=move || section.get() == item
                            aria-selected=move || {
                                if section.get() == item { "true" } else { "false" }
                            }
                            href=href
                            title=item.hint()
                            on:click=move |event| {
                                if let Some(on_navigate) = on_navigate {
                                    event.prevent_default();
                                    on_navigate.run(item);
                                }
                            }
                        >
                            {item.label()}
                        </a>
                    }
                }
            </For>
        </div>
    }
}
