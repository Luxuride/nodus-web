//! Moderation: decisions about people — reports, accounts, approvals, blocked domains.
//!
//! Administration (`/admin`) is the other half: the dashboard, relays, allowed domains and
//! announcements. The split follows Mastodon's own, and follows the reader: a moderator
//! looking for a report should not have to read server levers on the way.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::router::{self, Route};
use crate::state::alive::Alive;
use crate::state::bridge::admin;
use crate::ui::icons::Icon;
use crate::ui::page::PageHeader;

use self::accounts::AccountList;
use self::federation::Federation;
use self::registrations::Registrations;
use self::reports::ReportQueue;

use super::admin::StaffGate;

mod accounts;
mod federation;
mod registrations;
mod reports;

/// One section of the moderation area.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    Reports,
    Accounts,
    Registrations,
    Federation,
}

impl Section {
    pub const ALL: [Self; 4] = [
        Self::Reports,
        Self::Accounts,
        Self::Registrations,
        Self::Federation,
    ];

    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Reports => "reports",
            Self::Accounts => "accounts",
            Self::Registrations => "registrations",
            Self::Federation => "federation",
        }
    }

    #[must_use]
    pub fn of(slug: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|section| section.slug() == slug)
            .unwrap_or(Self::Reports)
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Reports => "Reports",
            Self::Accounts => "Accounts",
            Self::Registrations => "Registrations",
            Self::Federation => "Federation",
        }
    }

    #[must_use]
    pub const fn hint(self) -> &'static str {
        match self {
            Self::Reports => "Posts someone reported, and what was decided",
            Self::Accounts => "Look an account up, and act on it",
            Self::Registrations => "Who is waiting, and how somebody may join",
            Self::Federation => "Domains this instance refuses",
        }
    }

    #[must_use]
    pub const fn icon(self) -> Icon {
        match self {
            Self::Reports => Icon::Alert,
            Self::Accounts => Icon::Users,
            Self::Registrations => Icon::Check,
            Self::Federation => Icon::Shield,
        }
    }
}

/// The moderation area for one section.
#[expect(
    clippy::needless_pass_by_value,
    reason = "a Leptos component receives its props by value"
)]
#[component]
pub fn Moderation(section: String) -> impl IntoView {
    let section = StoredValue::new(Section::of(&section));
    let alive = Alive::new();
    // `None` is "not asked yet", `Some(NeedsGrant)` is "this token lacks the scope", and the
    // gate answers each of those differently: a spinner, a grant prompt, or the screens.
    let (allowed, set_allowed) = signal(None::<admin::Staff>);
    let probe = {
        move || {
            spawn_local(async move {
                let verdict = admin::staff().await;
                alive.if_alive(|| set_allowed.set(Some(verdict)));
            });
        }
    };
    Effect::new(move |_| probe());
    let retry = Callback::new(move |()| {
        set_allowed.set(None);
        probe();
    });

    view! {
        <section class="mx-auto flex w-full max-w-3xl flex-col gap-4 px-4 py-6">
            <PageHeader
                title="Moderation"
                description="Decisions about people: reports, accounts, registrations and blocked domains."
            />

            <StaffGate allowed=allowed retry=retry/>

            <Show when=move || allowed.get() == Some(admin::Staff::Yes)>
                <Strip current=section.get_value()/>
                {move || match section.get_value() {
                    Section::Reports => view! { <ReportQueue/> }.into_any(),
                    Section::Accounts => view! { <AccountList/> }.into_any(),
                    Section::Registrations => view! { <Registrations/> }.into_any(),
                    Section::Federation => view! { <Federation/> }.into_any(),
                }}
            </Show>
        </section>
    }
}

/// The section strip, one link each.
#[component]
fn Strip(current: Section) -> impl IntoView {
    view! {
        <div role="tablist" class="tabs tabs-box flex-wrap">
            <For each=|| Section::ALL key=|section| section.slug() let:kind>
                <a
                    role="tab"
                    class="tab gap-2"
                    class:tab-active=kind == current
                    href=format!("/moderation/{}", kind.slug())
                    title=kind.hint()
                    on:click=move |event| {
                        router::link(event, Route::Moderation(kind.slug().to_owned()));
                    }
                >
                    <Icon icon=kind.icon() class="size-4"/>
                    <span>{kind.label()}</span>
                </a>
            </For>
        </div>
    }
}
