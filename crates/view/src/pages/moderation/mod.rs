//! The moderation area: one section at a time, behind the staff gate.
//!
//! The area and its four sections were drawn in the client, so the server had nothing to paint
//! for `/moderation/{section}` beyond a skeleton. The heading, the gate and the strip are one
//! tree now; the section's body arrives as `children`, because each body reads its own list
//! and a page may not fetch. The section enum lives here rather than in the client for the
//! same reason the strip does: it is the strip's data, and one definition keeps the slug in
//! the address and the word on the tab from drifting apart.

use leptos::prelude::*;

use crate::chrome::section_strip::SectionTab;
use crate::chrome::{SectionStrip, StaffGate, StaffStanding};
use crate::icons::Icon;
use crate::{PageHeader, Screen};

pub mod accounts;
pub mod federation;
pub mod registrations;
pub mod reports;

/// The people lists (`/accounts/{id}/followers` and the viewer's mutes and blocks) share this
/// directory with the staff sections they are not part of; the file keeps the path the tree
/// gives it.
#[path = "[section]/mod.rs"]
pub mod people;

/// One section of the moderation area.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    /// The report queue.
    Reports,
    /// Account standing.
    Accounts,
    /// Who is waiting to be let in.
    Registrations,
    /// Domains this instance refuses.
    Federation,
}

impl Section {
    /// Every section, in the order the strip shows them.
    pub const ALL: [Self; 4] = [
        Self::Reports,
        Self::Accounts,
        Self::Registrations,
        Self::Federation,
    ];

    /// The last path segment, which is also the strip's key.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Reports => "reports",
            Self::Accounts => "accounts",
            Self::Registrations => "registrations",
            Self::Federation => "federation",
        }
    }

    /// The section an address names; anything else is the queue.
    #[must_use]
    pub fn of(slug: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|section| section.slug() == slug)
            .unwrap_or(Self::Reports)
    }

    /// The word on the tab.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Reports => "Reports",
            Self::Accounts => "Accounts",
            Self::Registrations => "Registrations",
            Self::Federation => "Federation",
        }
    }

    /// The sentence on hover: what the section is for.
    #[must_use]
    pub const fn hint(self) -> &'static str {
        match self {
            Self::Reports => "Posts someone reported, and what was decided",
            Self::Accounts => "Look an account up, and act on it",
            Self::Registrations => "Who is waiting, and how somebody may join",
            Self::Federation => "Domains this instance refuses",
        }
    }

    /// The drawing beside the label.
    #[must_use]
    pub const fn icon(self) -> Icon {
        match self {
            Self::Reports => Icon::Alert,
            Self::Accounts => Icon::Users,
            Self::Registrations => Icon::Check,
            Self::Federation => Icon::Shield,
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

/// The moderation area for one section.
///
/// The body is the caller's: each section reads its own list, and the page owns everything
/// around it — heading, gate, strip — so the order of those three cannot differ between the
/// paints.
#[expect(
    clippy::needless_pass_by_value,
    reason = "a Leptos component receives its props by value"
)]
#[component]
pub fn ModerationPage(
    /// The slug the address names.
    #[prop(into)]
    active: String,
    /// What the staff probe answered, or `None` while it is out.
    standing: Signal<Option<StaffStanding>>,
    /// Asks for the wider token, then retries the probe. Absent where nobody can ask.
    #[prop(optional)]
    on_grant: Option<Callback<()>>,
    /// Opens a section inside the app, by slug. Absent on the server.
    #[prop(optional)]
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
                title="Moderation"
                description="Decisions about people: reports, accounts, registrations and blocked domains."
            />

            <StaffGate standing=standing on_grant=on_grant>
                <SectionStrip
                    base="/moderation"
                    tabs=tabs.get_value()
                    active=here.slug()
                    on_navigate=on_navigate
                />
                {children()}
            </StaffGate>
        </Screen>
    }
}
