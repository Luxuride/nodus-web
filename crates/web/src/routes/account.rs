//! One account: who they are, whether we follow them, and what they posted.
//!
//! The relationship is a separate request from the account, because it depends on who
//! is asking: the same profile looks different to a follower, a stranger and someone who
//! blocked them.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::{Account, Relationship, Status};

use super::people::{Kind, PeopleList};
use super::status::StatusCard;
use crate::state::bridge::actions;
use tabs::AccountTabs;

mod profile;
mod tabs;

use self::profile::ProfileCard;

/// The three lists a profile offers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Posts,
    Followers,
    Following,
}

impl Tab {
    /// The tab label.
    fn label(self) -> &'static str {
        match self {
            Self::Posts => "Posts",
            Self::Followers => "Followers",
            Self::Following => "Following",
        }
    }
}

/// Which relationship verb a profile button sends.
#[derive(Debug, Clone, Copy)]
pub(super) enum Verb {
    Follow,
    Mute,
    Block,
}

impl Verb {
    /// The button label for this verb and the relationship as it stands.
    pub(super) fn label(self, relationship: Option<Relationship>) -> &'static str {
        match (self, relationship) {
            (Self::Follow, Some(found)) if found.following => "Unfollow",
            (Self::Mute, Some(found)) if found.muting => "Unmute",
            (Self::Block, Some(found)) if found.blocking => "Unblock",
            (Self::Follow, _) => "Follow",
            (Self::Mute, _) => "Mute",
            (Self::Block, _) => "Block",
        }
    }
}

/// A profile page.
#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "a Leptos component receives its props by value"
)]
pub fn AccountView(id: String) -> impl IntoView {
    let (account, set_account) = signal(None::<Account>);
    let (relationship, set_relationship) = signal(None::<Relationship>);
    let (tab, set_tab) = signal(Tab::Posts);
    let (statuses, set_statuses) = signal(Vec::<Status>::new());
    let (cursor, set_cursor) = signal(None::<String>);
    let (loading, set_loading) = signal(false);
    let (busy, set_busy) = signal(false);
    let (error, set_error) = signal(None::<String>);

    // `StoredValue` rather than a captured `String`: two `Show` bodies read the id, and
    // a value captured by move would make each of them `FnOnce`.
    let id_signal = StoredValue::new(id.clone());

    let account_id = id.clone();
    Effect::new(move |_| {
        // One clone per request: the three calls run in parallel and each owns its id.
        let for_account = account_id.clone();
        spawn_local(async move {
            match actions::account(&for_account).await {
                Ok(found) => set_account.set(Some(found)),
                Err(message) => set_error.set(Some(message)),
            }
        });
        let for_relationship = account_id.clone();
        spawn_local(async move {
            match actions::relationship(&for_relationship).await {
                Ok(Some(found)) => set_relationship.set(Some(found)),
                Ok(None) => {}
                Err(message) => set_error.set(Some(message)),
            }
        });
        let for_statuses = account_id.clone();
        spawn_local(async move {
            match actions::account_statuses(&for_statuses, None).await {
                Ok(page) => {
                    set_statuses.set(page.items);
                    set_cursor.set(page.next_max_id);
                }
                Err(message) => set_error.set(Some(message)),
            }
        });
    });

    // A `Callback` rather than a bare closure: it is read inside a `Show` body that must be
    // `Fn`, and a non-`Copy` handler would be moved out of the environment on first use.
    let more = {
        let id = id.clone();
        Callback::new(move |()| {
            set_loading.set(true);
            let id = id.clone();
            let after = cursor.get_untracked();
            spawn_local(async move {
                match actions::account_statuses(&id, after.as_deref()).await {
                    Ok(page) => {
                        set_statuses.update(|statuses| statuses.extend(page.items));
                        set_cursor.set(page.next_max_id);
                    }
                    Err(message) => set_error.set(Some(message)),
                }
                set_loading.set(false);
            });
        })
    };

    view! {
        <section class="flex flex-col gap-3">
            <ProfileCard
                account=account
                relationship=relationship
                set_relationship=set_relationship
                busy=busy
                set_busy=set_busy
                set_error=set_error
            />

            <AccountTabs tab=tab set_tab=set_tab/>

            <Show when=move || tab.get() == Tab::Posts>
                <ul class="flex flex-col gap-3">
                    <For each=move || statuses.get() key=|status| status.id.clone() let:status>
                        <StatusCard status=status/>
                    </For>
                </ul>

                <Show when=move || cursor.get().is_some()>
                    <button
                        class="btn btn-ghost btn-sm w-full"
                        disabled=move || loading.get()
                        on:click=move |_| more.run(())
                    >
                        "Load more"
                    </button>
                </Show>
            </Show>

            <Show when=move || tab.get() == Tab::Followers>
                <PeopleList id=id_signal.get_value() kind=Kind::Followers/>
            </Show>

            <Show when=move || tab.get() == Tab::Following>
                <PeopleList id=id_signal.get_value() kind=Kind::Following/>
            </Show>

            <Show when=move || error.get().is_some()>
                <p class="text-error text-xs">{move || error.get().unwrap_or_default()}</p>
            </Show>
        </section>
    }
}

/// Runs one relationship verb and stores whatever the instance answers.
///
/// A free function rather than a closure because three buttons share it: the signals are
/// `Copy`, so no handler ends up owning state that a rebuild would have to hand over
/// again.
pub(super) fn toggle(
    id: String,
    verb: Verb,
    relationship: ReadSignal<Option<Relationship>>,
    set_relationship: WriteSignal<Option<Relationship>>,
    set_busy: WriteSignal<bool>,
    set_error: WriteSignal<Option<String>>,
) {
    let on = match verb {
        Verb::Follow => !relationship
            .get_untracked()
            .is_some_and(|found| found.following),
        Verb::Mute => !relationship
            .get_untracked()
            .is_some_and(|found| found.muting),
        Verb::Block => !relationship
            .get_untracked()
            .is_some_and(|found| found.blocking),
    };
    set_busy.set(true);
    set_error.set(None);
    spawn_local(async move {
        let answer = match verb {
            Verb::Follow => actions::follow(&id, on).await,
            Verb::Mute => actions::mute(&id, on).await,
            Verb::Block => actions::block(&id, on).await,
        };
        match answer {
            Ok(found) => set_relationship.set(Some(found)),
            Err(message) => set_error.set(Some(message)),
        }
        set_busy.set(false);
    });
}
