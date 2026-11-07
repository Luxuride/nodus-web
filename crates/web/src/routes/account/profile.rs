//! The profile card: who they are and what can be done about it.
//!
//! Separated from [`super::AccountView`] because the card is the part that grows: every
//! relationship verb and every profile affordance lands here, while the page around it
//! stays a list of statuses.

use leptos::prelude::*;
use nodus_instance::model::{Account, Relationship};

use super::{Verb, toggle};
use crate::routes::feed::posts::plain_text;
use crate::routes::format::display_name;

/// A profile header with Follow, Mute and Block.
#[component]
pub(super) fn ProfileCard(
    account: ReadSignal<Option<Account>>,
    relationship: ReadSignal<Option<Relationship>>,
    set_relationship: WriteSignal<Option<Relationship>>,
    busy: ReadSignal<bool>,
    set_busy: WriteSignal<bool>,
    set_error: WriteSignal<Option<String>>,
) -> impl IntoView {
    view! {
        {move || {
            account.get().map(|profile| {
                let id = profile.id.clone();
                let heading = display_name(&profile.display_name, &profile.acct);
                let bio = plain_text(&profile.note);
                let link = format!("/accounts/{id}");
                let avatar = profile.avatar.clone();
                let acct = profile.acct.clone();
                let counts = format!(
                    "{} posts · {} following · {} followers",
                    profile.statuses_count, profile.following_count, profile.followers_count,
                );
                // One clone per button: each handler owns its id, so the card can rebuild
                // when the relationship changes without any handler being consumed.
                let follow_id = id.clone();
                let mute_id = id.clone();
                let block_id = id;
                view! {
                    <div class="card border border-base-300 bg-base-100 shadow-sm">
                        <div class="card-body gap-3 p-4">
                            <div class="flex items-start gap-3">
                                <a class="avatar" href=link>
                                    <div class="w-16 rounded-full bg-base-300">
                                        <img src=avatar alt="" referrerpolicy="no-referrer"/>
                                    </div>
                                </a>
                                <div class="min-w-0 flex-1">
                                    <p class="truncate text-lg font-semibold">{heading}</p>
                                    <p class="truncate text-sm opacity-60">{format!("@{acct}")}</p>
                                    <p class="mt-2 text-xs opacity-60">{counts}</p>
                                </div>
                            </div>
                            <p class="whitespace-pre-line text-sm leading-relaxed">{bio}</p>
                            <div class="flex flex-wrap items-center gap-2">
                                <button
                                    class="btn btn-primary btn-sm"
                                    disabled=move || busy.get()
                                    on:click=move |_| {
                                        toggle(
                                            follow_id.clone(),
                                            Verb::Follow,
                                            relationship,
                                            set_relationship,
                                            set_busy,
                                            set_error,
                                        );
                                    }
                                >
                                    {move || Verb::Follow.label(relationship.get())}
                                </button>
                                <button
                                    class="btn btn-ghost btn-sm"
                                    disabled=move || busy.get()
                                    on:click=move |_| {
                                        toggle(
                                            mute_id.clone(),
                                            Verb::Mute,
                                            relationship,
                                            set_relationship,
                                            set_busy,
                                            set_error,
                                        );
                                    }
                                >
                                    {move || Verb::Mute.label(relationship.get())}
                                </button>
                                <button
                                    class="btn btn-ghost btn-sm"
                                    disabled=move || busy.get()
                                    on:click=move |_| {
                                        toggle(
                                            block_id.clone(),
                                            Verb::Block,
                                            relationship,
                                            set_relationship,
                                            set_busy,
                                            set_error,
                                        );
                                    }
                                >
                                    {move || Verb::Block.label(relationship.get())}
                                </button>
                            </div>
                        </div>
                    </div>
                }
            })
        }}
    }
}
