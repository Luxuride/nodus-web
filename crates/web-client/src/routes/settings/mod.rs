//! Settings: the instance this browser holds a session for, the sync consent, and
//! sign-out.
//!
//! Background sync lives in [`sync`]: it is the one card here that talks to the
//! server, and keeping it beside the session card would make this screen the place
//! where two unrelated questions are answered.

use leptos::prelude::*;
use nodus_pages::pages::settings::nav::SettingsSection;

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
use tags::FeaturedTagsCard;

/// One settings section's body.
#[component]
pub fn SettingsBody(section: SettingsSection) -> impl IntoView {
    let revision = revision();

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
            <Show when=move || section == SettingsSection::Account>
                <SessionCard host=host acct=acct/>
                <AccountsCard/>
                <ModerationLinks/>
            </Show>

            <Show when=move || section == SettingsSection::Profile>
                <ProfileCard/>
                <ClockCard/>
                <SensitiveCard/>
            </Show>

            <Show when=move || section == SettingsSection::Nodus>
                <SyncCard/>
                <FeedsCard/>
            </Show>

            <Show when=move || section == SettingsSection::Filters>
                <FiltersCard/>
                <FeaturedTagsCard/>
            </Show>

            <Show when=move || section == SettingsSection::Instance>
                <InstanceCard/>
            </Show>
    }
}
