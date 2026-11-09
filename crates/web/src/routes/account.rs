//! One account: who they are, whether we follow them, and what they posted.
//!
//! The relationship is a separate request from the account, because it depends on who
//! is asking: the same profile looks different to a follower, a stranger and someone who
//! blocked them.
//!
//! The paint is the shared page in `nodus-view`; this module owns the state and the
//! requests, and supplies the two things the page may not know: the tab strip, and the foot
//! where the next page and the other tabs sit.

use std::sync::Arc;

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::{Account, Relationship};
use nodus_sdk::Timeline;

use super::people::{Kind, PeopleList};
use crate::state::alive::Alive;
use crate::state::bridge::actions;
use crate::state::bridge::revision;
use crate::state::bridge::timeline;
use tabs::AccountTabs;

mod profile;
mod tabs;

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
    // `StoredValue` rather than a captured `Timeline`: the handle is `Copy`, so the
    // reader closure stays `Copy` too and can be used in two places in one `view!`.
    let which = StoredValue::new(Timeline::Account(id.clone()));
    let posts = move || {
        revision().get();
        timeline::read(which.get_value(), None)
    };
    // Starts true: until the first request is issued, "nothing yet" is a
    // guess, and a guess renders as an empty state that flickers.
    let (loading, set_loading) = signal(true);
    let (busy, set_busy) = signal(false);
    let (error, set_error) = signal(None::<String>);
    let alive = Alive::new();

    // `StoredValue` rather than a captured `String`: two `Show` bodies read the id, and
    // a value captured by move would make each of them `FnOnce`.
    let id_signal = StoredValue::new(id.clone());

    let account_id = id.clone();
    Effect::new(move |_| {
        load_profile(
            &account_id,
            alive,
            set_account,
            set_relationship,
            set_loading,
            set_error,
        );
    });

    // A `Callback` rather than a bare closure: it is read inside a `Show` body that must be
    // `Fn`, and a non-`Copy` handler would be moved out of the environment on first use.
    let more = Callback::new(move |()| {
        set_loading.set(true);
        let which = which.get_value();
        let after = posts().1;

        spawn_local(async move {
            match timeline::ensure(which, None, after).await {
                Ok(_) => alive.if_alive(|| set_loading.set(false)),
                Err(message) => alive.if_alive(|| {
                    set_error.set(Some(message));
                    set_loading.set(false);
                }),
            }
        });
    });

    // The page reads the list as a signal, so a page of posts lands in the list without the
    // header being rebuilt. The cursor travels with it, for the foot's "Load more".
    let statuses = Signal::derive(move || posts().0);
    let next = Signal::derive(move || posts().1);

    // The tab strip is the client's: it switches a list, not a page. It is a slot so that
    // the page can keep the header above it and the list below, which is the one order both
    // paints agree on.
    let tabs: ChildrenFn =
        Arc::new(move || view! { <AccountTabs tab=tab set_tab=set_tab/> }.into_any());

    // What sits under the list: the next page, or the other tab's list, or the error a
    // request brought back. The page owns the status list and nothing below it.
    let after: ChildrenFn = Arc::new(move || {
        view! {
            <Show when=move || tab.get() == Tab::Posts>
                <Show when=move || next.get().is_some()>
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
        }
        .into_any()
    });

    // A verb pressed while another is in flight is ignored rather than disabled: the shared
    // page decides `disabled` from the relationship alone, and the labels do not flicker.
    let verbs = verbs(alive, busy, relationship, set_relationship, set_busy, set_error);

    let on_avatar = Callback::new(|url: String| crate::media::expand_one(url, String::new()));

    // The page is built once the account has arrived: until then there is no header to
    // draw, and a header drawn from a guess is the flicker this module exists to avoid.
    view! {
        {move || {
            account.get().map(|profile| {
                view! {
                    <nodus_view::profile::ProfilePage
                        account=profile
                        relationship=relationship.get()
                        statuses=statuses
                        show_statuses=tab.get() == Tab::Posts
                        tabs=tabs.clone()
                        after=after.clone()
                        card=profile::card()
                        age_of=Callback::new(|created_at: String| profile::age(&created_at))
                        on_follow=verbs.follow
                        on_mute=verbs.mute
                        on_block=verbs.block
                        on_avatar=on_avatar
                    />
                }
            })
        }}
    }
}

/// The three reads a profile needs, issued together.
///
/// They are one function so the page reads as a page: the profile, the relationship that
/// decides which verbs make sense, and its first page of posts. Each call owns its own clone
/// of the id, because they run in parallel and a shared `&str` cannot outlive the first spawn.
fn load_profile(
    id: &str,
    alive: Alive,
    set_account: WriteSignal<Option<Account>>,
    set_relationship: WriteSignal<Option<Relationship>>,
    set_loading: WriteSignal<bool>,
    set_error: WriteSignal<Option<String>>,
) {
    let for_account = id.to_owned();
    spawn_local(async move {
        match actions::account(&for_account).await {
            Ok(found) => alive.if_alive(|| set_account.set(Some(found))),
            Err(message) => alive.if_alive(|| set_error.set(Some(message))),
        }
    });

    let for_relationship = id.to_owned();
    spawn_local(async move {
        match actions::relationship(&for_relationship).await {
            Ok(Some(found)) => alive.if_alive(|| set_relationship.set(Some(found))),
            Ok(None) => {}
            Err(message) => alive.if_alive(|| set_error.set(Some(message))),
        }
    });

    let for_posts = Timeline::Account(id.to_owned());
    spawn_local(async move {
        match timeline::ensure(for_posts, None, None).await {
            Ok(_) => alive.if_alive(|| set_loading.set(false)),
            Err(message) => alive.if_alive(|| {
                set_error.set(Some(message));
                set_loading.set(false);
            }),
        }
    });
}

/// The profile's three relationship verbs, drawn once and shared by the buttons.
#[derive(Clone, Copy)]
struct Verbs {
    follow: Callback<(String, bool)>,
    mute: Callback<(String, bool)>,
    block: Callback<(String, bool)>,
}

/// Builds the three verbs around one guard.
///
/// A verb pressed while another is in flight is ignored rather than disabled: the shared page
/// decides `disabled` from the relationship alone, and the labels must not flicker.
fn verbs(
    alive: Alive,
    busy: ReadSignal<bool>,
    relationship: ReadSignal<Option<Relationship>>,
    set_relationship: WriteSignal<Option<Relationship>>,
    set_busy: WriteSignal<bool>,
    set_error: WriteSignal<Option<String>>,
) -> Verbs {
    let one = |verb: Verb| {
        Callback::new(move |(id, on): (String, bool)| {
            if busy.get_untracked() {
                return;
            }
            toggle(
                id,
                verb,
                on,
                alive,
                relationship,
                set_relationship,
                set_busy,
                set_error,
            );
        })
    };
    Verbs {
        follow: one(Verb::Follow),
        mute: one(Verb::Mute),
        block: one(Verb::Block),
    }
}

/// Runs one relationship verb and stores whatever the instance answers.
///
/// A free function rather than a closure because three buttons share it: the signals are
/// `Copy`, so no handler ends up owning state that a rebuild would have to hand over
/// again. The flag comes from the page, which knows the relationship the buttons were drawn
/// against; this only refuses to send when the relationship was never answered at all.
#[expect(
    clippy::too_many_arguments,
    reason = "the template's action set: id, verb, target state, and the three signals it writes"
)]
pub(super) fn toggle(
    id: String,
    verb: Verb,
    on: bool,
    alive: Alive,
    relationship: ReadSignal<Option<Relationship>>,
    set_relationship: WriteSignal<Option<Relationship>>,
    set_busy: WriteSignal<bool>,
    set_error: WriteSignal<Option<String>>,
) {
    // Acting on an unknown relationship would guess the verb: with no answer yet we
    // cannot tell whether the button means "follow" or "unfollow".
    if relationship.get_untracked().is_none() {
        return;
    }
    set_busy.set(true);
    set_error.set(None);
    spawn_local(async move {
        let answer = match verb {
            Verb::Follow => actions::follow(&id, on).await,
            Verb::Mute => actions::mute(&id, on).await,
            Verb::Block => actions::block(&id, on).await,
        };
        match answer {
            Ok(found) => alive.if_alive(|| set_relationship.set(Some(found))),
            Err(message) => alive.if_alive(|| set_error.set(Some(message))),
        }
        alive.if_alive(|| set_busy.set(false));
    });
}
