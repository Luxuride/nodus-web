//! Settings: the instance this browser holds a session for, the sync consent, and
//! sign-out.
//!
//! Background sync lives in [`sync`]: it is the one card here that talks to the
//! server, and keeping it beside the session card would make this screen the place
//! where two unrelated questions are answered.

use leptos::prelude::*;

use crate::routes::settings::sync::SyncCard;
use crate::state::bridge::{revision, with_sdk};

mod accounts;

/// The grant prompt. Moderation shows it in place of its own message when the token lacks
/// the admin scopes, so it lives here rather than being duplicated — but it only exists in a
/// build that carries the moderation screens.
#[cfg(feature = "admin")]
pub(crate) mod admin;
pub(crate) mod clock;
mod feeds;
pub(crate) mod filters;
pub(crate) mod instance;
mod moderation;
mod profile;
pub(crate) mod sensitive;
mod session;
pub(crate) mod sync;
mod tabs;
pub(crate) mod tags;

use accounts::AccountsCard;
use clock::ClockCard;
use feeds::FeedsCard;
use filters::FiltersCard;
use instance::InstanceCard;
use moderation::ModerationLinks;
use profile::ProfileCard;
use sensitive::SensitiveCard;
use session::SessionCard;
use tabs::{SettingsTabs, Tab};
use tags::FeaturedTagsCard;

/// The section a `/settings/{slug}` address names.
/// A Leptos component receives its props by value; the body only reads it.
#[expect(
    clippy::needless_pass_by_value,
    reason = "a Leptos component receives its props by value"
)]
#[component]
pub fn Settings(section: String) -> impl IntoView {
    let revision = revision();
    // The address decides the section: `/settings/privacy` opens Privacy.
    let (tab, _set_tab) = signal(Tab::of(&section));

    let host = move || {
        revision.get();
        with_sdk(|sdk| {
            sdk.state()
                .session
                .get()
                .map_or_else(String::new, |session| session.host)
        })
    };
    let acct = move || {
        revision.get();
        with_sdk(|sdk| {
            sdk.state()
                .session
                .get()
                .map_or_else(String::new, |session| session.acct)
        })
    };

    view! {
        <section class="mx-auto w-full max-w-2xl px-4 py-6">
            <h1 class="mb-5 text-xl font-semibold tracking-tight">"Settings"</h1>

            <SettingsTabs tab=tab/>

            <Show when=move || tab.get() == Tab::Account>
                <SessionCard host=host acct=acct/>
                <AccountsCard/>
                <ModerationLinks/>
            </Show>

            <Show when=move || tab.get() == Tab::Profile>
                <ProfileCard/>
                <ClockCard/>
                <SensitiveCard/>
            </Show>

            <Show when=move || tab.get() == Tab::Nodus>
                <SyncCard/>
                <FeedsCard/>
            </Show>

            <Show when=move || tab.get() == Tab::Filters>
                <FiltersCard/>
                <FeaturedTagsCard/>
            </Show>

            <Show when=move || tab.get() == Tab::Instance>
                <InstanceCard/>
            </Show>


        </section>
    }
}
