//! One ranked account: who it is, why it is here, and what can be done with it.

use leptos::prelude::*;
use nodus_shared::AccountItem;

use super::actions::{ItemActions, item_actions};
use super::reason::{follower_label, reason_line, reason_title, sample_followers_line};
use super::share::ShareReport;
use crate::state::alive::Alive;
use crate::state::bridge::{begin_impression, finish_impression};

/// One ranked account: who it is, why it is here, and the three actions.
///
/// The card is the whole product in one unit, so everything a user needs to decide
/// sits on it: identity, the reason (plus its samples), and Follow / Not interested /
/// Copy link. The parent keys the list on the follow flag, so undoing a follow
/// remounts this card instead of leaving a stale "Following" label.
#[component]
pub(super) fn FeedItem(item: AccountItem) -> impl IntoView {
    let AccountItem {
        item_id,
        account,
        reason: why,
    } = item;
    let account_id = account.id;
    let acct = account.acct;
    let display_name = if account.display_name.trim().is_empty() {
        acct.clone()
    } else {
        account.display_name
    };
    let note = account.note;
    let avatar_url = account.avatar_url;
    let followers = follower_label(account.followers_count);
    let already_following = account.already_following;

    let (reason, reason_hint, samples) = (
        reason_line(&why),
        reason_title(&why),
        sample_followers_line(&why),
    );

    // D36: the impression is written when the card leaves the screen, carrying the
    // time it spent there. Beginning twice is harmless — the tracker keeps the
    // earliest start — so a re-render cannot inflate the number.
    begin_impression(item_id);
    on_cleanup(move || finish_impression(item_id));

    // The card can be unmounted by a follow flip while one of its requests is in
    // flight, so the handlers need the flag rather than the signals alone.
    let alive = Alive::new();

    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let toast = RwSignal::new(None::<ShareReport>);
    let ItemActions {
        follow,
        unfollow,
        not_interested,
        copy,
    } = item_actions(item_id, account_id, acct.clone(), busy, error, toast, alive);

    // One button instead of two `view!` branches: the difference is a label, a class
    // and which handler runs, and three lines of metadata beat twenty of duplication.
    let (follow_label, follow_class) = if already_following {
        ("Following", "btn btn-ghost btn-sm")
    } else {
        ("Follow", "btn btn-primary btn-sm")
    };
    let mut follow = follow;
    let mut unfollow = unfollow;
    let follow_click = move |event: leptos::ev::MouseEvent| {
        if already_following {
            unfollow(event);
        } else {
            follow(event);
        }
    };

    view! {
        <li class="card border border-base-300 bg-base-100 shadow-sm">
            <div class="card-body gap-3 p-4">
                <Identity
                    display_name=display_name.clone()
                    acct=acct.clone()
                    avatar_url=avatar_url.clone()
                    followers=followers.clone()
                />
                // The reason is the product: an item the user cannot explain is an
                // item they will not trust. It is a badge, not body text, so it reads
                // as metadata rather than as the account's own description; the
                // hover text carries the raw signal for anyone who wants it.
                <ReasonBlock reason=reason.clone() hint=reason_hint samples=samples note=note.clone() />
                <div class="card-actions justify-end gap-2">
                    <button
                        class="btn btn-ghost btn-sm"
                        title="Copy profile link"
                        aria-label="Copy profile link"
                        on:click=copy
                    >
                        "Copy link"
                    </button>
                    <button
                        class="btn btn-ghost btn-sm"
                        disabled=move || busy.get()
                        on:click=not_interested
                    >
                        "Not interested"
                    </button>
                    <button class=follow_class disabled=move || busy.get() on:click=follow_click>
                        <Show when=move || busy.get() && !already_following>
                            <span class="loading loading-spinner loading-xs"></span>
                        </Show>
                        {follow_label}
                    </button>
                </div>
                {move || error.get().map(|message| view! {
                    <p class="text-sm text-error" role="alert">{message}</p>
                })}
                <CopyToast toast=toast />
            </div>
        </li>
    }
}

/// Who the account is: avatar, name, handle, and how big it is.
///
/// Split out of [`FeedItem`] because the card's markup was long enough to hide the
/// behaviour in it; this component holds no state, so it is also the part a test
/// could snapshot.
#[component]
fn Identity(
    display_name: String,
    acct: String,
    avatar_url: String,
    followers: String,
) -> impl IntoView {
    view! {
        <div class="flex items-start justify-between gap-3">
            <div class="flex min-w-0 items-center gap-3">
                <div class="avatar">
                    <div class="w-10 rounded-full bg-base-300">
                        <img src=avatar_url alt="" referrerpolicy="no-referrer" />
                    </div>
                </div>
                <div class="min-w-0">
                    <p class="truncate font-semibold leading-tight">{display_name}</p>
                    <p class="truncate text-sm opacity-60">{format!("@{acct}")}</p>
                </div>
            </div>
            <span class="badge badge-ghost badge-sm whitespace-nowrap">{followers}</span>
        </div>
    }
}

/// Why the item is on the page: the badge, its hover text, the sample followers,
/// and the account's own description when there is one.
#[component]
fn ReasonBlock(
    reason: String,
    hint: String,
    samples: Option<String>,
    note: String,
) -> impl IntoView {
    view! {
        <div class="space-y-1">
            <span class="badge badge-soft badge-primary badge-sm" title=hint>
                {reason}
            </span>
            {samples.map(|line| view! { <p class="text-xs opacity-70">{line}</p> })}
        </div>
        {(!note.trim().is_empty()).then(|| view! {
            <p class="line-clamp-3 text-sm opacity-80">{note}</p>
        })}
    }
}

/// The transient "profile link copied" report, or nothing.
///
/// A toast rather than an inline message because the card may unmount at any moment
/// (a follow flips the list key), and a report that vanishes with its card is not a
/// report.
#[component]
fn CopyToast(toast: RwSignal<Option<ShareReport>>) -> impl IntoView {
    view! {
        {move || toast.get().map(|report| {
            let tone = if report.ok {
                "alert alert-success alert-soft text-sm"
            } else {
                "alert alert-error alert-soft text-sm"
            };
            view! {
                <div class="toast toast-end" role="status">
                    <div class=tone>{report.message}</div>
                </div>
            }
        })}
    }
}
