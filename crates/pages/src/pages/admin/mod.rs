//! Administration: the instance itself — what it counted, who it talks to, what it says.
//!
//! Distinct from Moderation on purpose. Moderating is a decision about people (reports,
//! accounts, approvals, blocked domains); administering is a decision about the server (the
//! dashboard, relays, allowed domains, announcements). Mastodon draws the same line, and a
//! moderator who is not an administrator should not have to read server levers to find the
//! report queue.
//!
//! The area — heading, gate, strip — is one tree for both paints; each section's body arrives
//! as `children`, because a body reads its own numbers and a page may not fetch.

use leptos::prelude::*;

use crate::chrome::section_strip::SectionTab;
use crate::chrome::{SectionStrip, StaffGate, StaffStanding};
use crate::icons::Icon;
use crate::{PageHeader, Screen};

pub mod announcements;
pub mod audit;
pub mod domains;
pub mod overview;
pub mod protocol;

/// One section of the administration area.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    /// The dashboard.
    Overview,
    /// Domains allowed through a general block.
    Domains,
    /// What the instance is telling everyone.
    Announcements,
    /// What staff have done.
    Audit,
    /// What API this server publishes.
    Protocol,
}

impl Section {
    /// Every section, in the order the strip shows them.
    pub const ALL: [Self; 5] = [
        Self::Overview,
        Self::Domains,
        Self::Announcements,
        Self::Audit,
        Self::Protocol,
    ];

    /// The last path segment, which is also the strip's key.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Overview => "overview",
            Self::Domains => "domains",
            Self::Announcements => "announcements",
            Self::Audit => "audit",
            Self::Protocol => "protocol",
        }
    }

    /// The section an address names; anything else is the dashboard.
    #[must_use]
    pub fn of(slug: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|section| section.slug() == slug)
            .unwrap_or(Self::Overview)
    }

    /// The word on the tab.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Domains => "Allowed domains",
            Self::Announcements => "Announcements",
            Self::Audit => "Audit log",
            Self::Protocol => "Protocol",
        }
    }

    /// The sentence on hover: what the section is for.
    #[must_use]
    pub const fn hint(self) -> &'static str {
        match self {
            Self::Overview => "What this instance counted this week",
            Self::Domains => "Domains allowed through a general block",
            Self::Announcements => "What the instance is telling everyone",
            Self::Audit => "Administrative actions recorded by this server",
            Self::Protocol => "The client API the SDK selects for this server",
        }
    }

    /// The drawing beside the label.
    #[must_use]
    pub const fn icon(self) -> Icon {
        match self {
            Self::Overview | Self::Protocol => Icon::Globe,
            Self::Domains => Icon::Link,
            Self::Announcements => Icon::Bell,
            Self::Audit => Icon::List,
        }
    }

    /// The tab the strip draws for this section.
    #[must_use]
    pub fn tab(self) -> SectionTab {
        SectionTab {
            slug: self.slug().to_owned(),
            label: self.label().to_owned(),
            hint: self.hint().to_owned(),
            icon: self.icon(),
        }
    }
}

/// The administration area for one section.
#[expect(
    clippy::needless_pass_by_value,
    reason = "a Leptos component receives its props by value"
)]
#[component]
pub fn AdminPage(
    /// The slug the address names.
    #[prop(into)]
    active: String,
    /// What the staff probe answered, or `None` while it is out.
    standing: Signal<Option<StaffStanding>>,
    /// Asks for the wider token, then retries the probe. Absent where nobody can ask.
    #[prop(optional_no_strip)]
    on_grant: Option<Callback<()>>,
    /// Opens a section inside the app, by slug. Absent on the server.
    #[prop(optional_no_strip)]
    on_navigate: Option<Callback<String>>,
    /// The section body.
    children: ChildrenFn,
) -> impl IntoView {
    let here = Section::of(&active);
    // `StoredValue`: the strip is drawn inside the gate's `ChildrenFn`, which must be
    // `Fn`, and a `Vec` moved into that closure would make it `FnOnce`.
    let tabs = StoredValue::new(
        Section::ALL
            .iter()
            .map(|section| section.tab())
            .collect::<Vec<SectionTab>>(),
    );

    view! {
        <Screen>
            <PageHeader
                title="Administration"
                description="This instance: what it counted, who it talks to, what it says."
            />

            <StaffGate standing=standing on_grant=on_grant>
                <SectionStrip
                    base="/admin"
                    tabs=tabs.get_value()
                    active=here.slug()
                    on_navigate=on_navigate
                />
                {children()}
            </StaffGate>
        </Screen>
    }
}
