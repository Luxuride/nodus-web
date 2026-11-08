//! Settings: the instance this browser holds a session for, the sync consent, and
//! sign-out.
//!
//! Background sync lives in [`sync`]: it is the one card here that talks to the
//! server, and keeping it beside the session card would make this screen the place
//! where two unrelated questions are answered.

use leptos::prelude::*;

use crate::router::{self, Route};
use crate::routes::settings::sync::SyncCard;
use crate::state::bridge::{revision, sign_out, with_sdk};

mod accounts;
mod admin;
pub(crate) mod clock;
mod feeds;
pub(crate) mod filters;
pub(crate) mod instance;
mod profile;
pub(crate) mod sensitive;
pub(crate) mod sync;
mod tabs;
pub(crate) mod tags;

use accounts::AccountsCard;
use admin::AdministrationCard;
use clock::ClockCard;
use feeds::FeedsCard;
use filters::FiltersCard;
use instance::InstanceCard;
use profile::ProfileCard;
use sensitive::SensitiveCard;
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
                            <div class="card mb-4 border border-base-300 bg-base-100 shadow-sm">
                <div class="card-body gap-2 p-4">
                    <h2 class="card-title text-base">"Session"</h2>
                    <dl class="grid grid-cols-[8rem_minmax(0,1fr)] gap-x-3 gap-y-1 text-sm">
                        <dt class="opacity-60">"Instance"</dt>
                        <dd class="truncate">{move || host()}</dd>
                        <dt class="opacity-60">"Account"</dt>
                        <dd class="truncate">{move || acct()}</dd>
                    </dl>
                    <div class="card-actions mt-2 justify-end">
                        <button class="btn btn-ghost btn-sm" on:click=move |_| sign_out()>
                            "Sign out"
                        </button>
                    </div>
                    <p class="text-xs opacity-60">"Signing out forgets the token stored in this browser."</p>
                </div>
            </div>

                <AccountsCard/>
                            <div class="card mt-4 border border-base-300 bg-base-100 shadow-sm">
                <div class="card-body gap-2 p-4">
                    <h2 class="card-title text-base">"Moderation"</h2>
                    <p class="text-sm opacity-70">
                        "Muting hides an account from your timelines; blocking also stops it following you."
                    </p>
                    <div class="card-actions mt-1 gap-2">
                        <a
                            class="btn btn-ghost btn-sm"
                            href=Route::Admin.path()
                            on:click=move |event| router::link(event, Route::Admin)
                        >
                            "Instance moderation"
                        </a>
                        <a
                            class="btn btn-ghost btn-sm"
                            href=Route::Mutes.path()
                            on:click=move |event| router::link(event, Route::Mutes)
                        >
                            "Muted accounts"
                        </a>
                        <a
                            class="btn btn-ghost btn-sm"
                            href=Route::Blocks.path()
                            on:click=move |event| router::link(event, Route::Blocks)
                        >
                            "Blocked accounts"
                        </a>
                    </div>
                </div>
            </div>
            </Show>

            <Show when=move || tab.get() == Tab::Profile>
                <ProfileCard/>
                <ClockCard/>
                <SensitiveCard/>
            </Show>

            <Show when=move || tab.get() == Tab::Privacy>
                <SyncCard/>
            </Show>

            <Show when=move || tab.get() == Tab::Feeds>
                <FeedsCard/>
            </Show>

            <Show when=move || tab.get() == Tab::Filters>
                <FiltersCard/>
            </Show>

            <Show when=move || tab.get() == Tab::Tags>
                <FeaturedTagsCard/>
            </Show>

            <Show when=move || tab.get() == Tab::Instance>
                <InstanceCard/>
            </Show>

            <Show when=move || tab.get() == Tab::Administration>
                <AdministrationCard/>
            </Show>

        </section>
    }
}
