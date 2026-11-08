//! Administration: one area, six sections, each its own address.
//!
//! The sections mirror what a moderator actually does — read the queue, look a person up,
//! decide who may join, steer federation, look at the week — rather than the endpoints.
//! Everything under `/admin/…` is gated on a probe: the token is the authority, and a
//! screen that renders controls which answer 403 teaches nothing.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::router::{self, Route};
use crate::state::alive::Alive;
use crate::state::bridge::admin;
use crate::ui::icons::Icon;
use crate::ui::page::PageHeader;

use self::accounts::AccountList;
use self::announcements::Announcements;
use self::federation::Federation;
use self::overview::Overview;
use self::registrations::Registrations;
use self::reports::ReportQueue;

mod accounts;
mod announcements;
mod federation;
mod overview;
mod registrations;
mod reports;

/// One section of the administration area.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    Overview,
    Reports,
    Accounts,
    Registrations,
    Federation,
    Announcements,
}

impl Section {
    pub const ALL: [Self; 6] = [
        Self::Overview,
        Self::Reports,
        Self::Accounts,
        Self::Registrations,
        Self::Federation,
        Self::Announcements,
    ];

    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Overview => "overview",
            Self::Reports => "reports",
            Self::Accounts => "accounts",
            Self::Registrations => "registrations",
            Self::Federation => "federation",
            Self::Announcements => "announcements",
        }
    }

    /// Unknown slugs fall back to the overview rather than 404: a stale bookmark should
    /// land somewhere useful.
    #[must_use]
    pub fn of(slug: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|section| section.slug() == slug)
            .unwrap_or(Self::Overview)
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Reports => "Reports",
            Self::Accounts => "Accounts",
            Self::Registrations => "Registrations",
            Self::Federation => "Federation",
            Self::Announcements => "Announcements",
        }
    }

    #[must_use]
    pub const fn hint(self) -> &'static str {
        match self {
            Self::Overview => "What this instance counted this week",
            Self::Reports => "Posts someone reported, and what was decided",
            Self::Accounts => "Look an account up, and act on it",
            Self::Registrations => "Who is waiting, and how somebody may join",
            Self::Federation => "Domains refused, relays used, domains allowed",
            Self::Announcements => "What the instance is telling everyone",
        }
    }

    #[must_use]
    pub const fn icon(self) -> Icon {
        match self {
            Self::Overview => Icon::Globe,
            Self::Reports => Icon::Alert,
            Self::Accounts => Icon::Users,
            Self::Registrations => Icon::Check,
            Self::Federation => Icon::Link,
            Self::Announcements => Icon::Bell,
        }
    }
}

/// The administration area for one section.
#[expect(
    clippy::needless_pass_by_value,
    reason = "a Leptos component receives its props by value"
)]
#[component]
pub fn Admin(section: String) -> impl IntoView {
    let section = StoredValue::new(Section::of(&section));
    let alive = Alive::new();
    let (allowed, set_allowed) = signal(None::<bool>);

    Effect::new(move |_| {
        spawn_local(async move {
            let verdict = matches!(admin::staff().await, admin::Staff::Yes);
            alive.if_alive(|| set_allowed.set(Some(verdict)));
        });
    });

    view! {
        <section class="mx-auto flex w-full max-w-3xl flex-col gap-4 px-4 py-6">
            <PageHeader
                title="Administration"
                description="Everything a moderator of this instance can do from here."
            />

            <Show when=move || allowed.get().is_none()>
                <p class="text-sm opacity-70">"Checking what this account may do…"</p>
            </Show>

            <Show when=move || allowed.get() == Some(false)>
                <div class="card border border-base-300 bg-base-100 shadow-sm">
                    <div class="card-body gap-2 p-4">
                        <h2 class="card-title text-base">"Not a moderator here"</h2>
                        <p class="text-sm opacity-70">
                            "This account cannot moderate this instance, so nothing below would
                             work anyway. Settings → Administration can ask for the permission
                             if you believe it should."
                        </p>
                        <a class="link link-primary w-fit text-sm" href="/settings/administration">
                            "Open Settings → Administration"
                        </a>
                    </div>
                </div>
            </Show>

            <Show when=move || allowed.get() == Some(true)>
                <Sections current=section.get_value()/>
                {move || match section.get_value() {
                    Section::Overview => view! { <Overview/> }.into_any(),
                    Section::Reports => view! { <ReportQueue/> }.into_any(),
                    Section::Accounts => view! { <AccountList/> }.into_any(),
                    Section::Registrations => view! { <Registrations/> }.into_any(),
                    Section::Federation => view! { <Federation/> }.into_any(),
                    Section::Announcements => view! { <Announcements/> }.into_any(),
                }}
            </Show>
        </section>
    }
}

/// The section strip. Each entry is a link, so a section can be bookmarked and shared.
#[component]
fn Sections(current: Section) -> impl IntoView {
    view! {
        <div role="tablist" class="tabs tabs-box flex-wrap">
            <For each=|| Section::ALL key=|section| section.slug() let:kind>
                <a
                    role="tab"
                    class="tab gap-2"
                    class:tab-active=kind == current
                    href=format!("/admin/{}", kind.slug())
                    title=kind.hint()
                    on:click=move |event| router::link(event, Route::Admin(kind.slug().to_owned()))
                >
                    <Icon icon=kind.icon() class="size-4"/>
                    <span>{kind.label()}</span>
                </a>
            </For>
        </div>
    }
}
