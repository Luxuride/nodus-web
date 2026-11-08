//! Administration: the instance itself — what it counted, who it talks to, what it says.
//!
//! Distinct from Moderation on purpose. Moderating is a decision about people (reports,
//! accounts, approvals, blocked domains); administering is a decision about the server
//! (the dashboard, relays, allowed domains, announcements). Mastodon draws the same line,
//! and a moderator who is not an administrator should not have to read server levers to
//! find the report queue.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::router::{self, Route};
use crate::state::alive::Alive;
use crate::state::bridge::admin;
use crate::ui::icons::Icon;
use crate::ui::page::PageHeader;

use self::announcements::Announcements;
use self::domains::Domains;
use self::overview::Overview;

mod announcements;
mod domains;
mod overview;

/// One section of the administration area.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    Overview,
    Domains,
    Announcements,
}

impl Section {
    pub const ALL: [Self; 3] = [Self::Overview, Self::Domains, Self::Announcements];

    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Overview => "overview",
            Self::Domains => "domains",
            Self::Announcements => "announcements",
        }
    }

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
            Self::Domains => "Allowed domains",
            Self::Announcements => "Announcements",
        }
    }

    #[must_use]
    pub const fn hint(self) -> &'static str {
        match self {
            Self::Overview => "What this instance counted this week",
            Self::Domains => "Domains allowed through a general block",
            Self::Announcements => "What the instance is telling everyone",
        }
    }

    #[must_use]
    pub const fn icon(self) -> Icon {
        match self {
            Self::Overview => Icon::Globe,
            Self::Domains => Icon::Link,
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
                description="This instance: what it counted, who it talks to, what it says."
            />

            <StaffGate allowed=allowed/>

            <Show when=move || allowed.get() == Some(true)>
                <Strip current=section.get_value()/>
                {move || match section.get_value() {
                    Section::Overview => view! { <Overview/> }.into_any(),
                    Section::Domains => view! { <Domains/> }.into_any(),
                    Section::Announcements => view! { <Announcements/> }.into_any(),
                }}
            </Show>
        </section>
    }
}

/// What to say while the probe is out, and when it comes back negative.
#[component]
pub fn StaffGate(allowed: ReadSignal<Option<bool>>) -> impl IntoView {
    view! {
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
    }
}

/// A strip of sections, each one a link, so a section can be bookmarked and shared.
#[component]
fn Strip(current: Section) -> impl IntoView {
    view! {
        <div role="tablist" class="tabs tabs-box flex-wrap">
            <For each=|| Section::ALL key=|section| section.slug() let:kind>
                <a
                    role="tab"
                    class="tab gap-2"
                    class:tab-active=kind == current
                    href=format!("/admin/{}", kind.slug())
                    title=kind.hint()
                    on:click=move |event| {
                        router::link(event, Route::Admin(kind.slug().to_owned()));
                    }
                >
                    <Icon icon=kind.icon() class="size-4"/>
                    <span>{kind.label()}</span>
                </a>
            </For>
        </div>
    }
}
