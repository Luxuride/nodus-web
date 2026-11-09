//! One ranked account: who it is, why it is here, and what can be done with it.

use leptos::prelude::*;
use nodus_shared::AccountItem;

use super::actions::{ItemActions, item_actions};
use super::reason::{follower_label, reason_line, reason_title, sample_followers_line};
use super::share::ShareReport;
use crate::state::alive::Alive;
use crate::state::bridge::{begin_impression, finish_impression};

use nodus_view::pages::feed::accounts::{Toast, ToastReport};

/// One ranked account: who it is, why it is here, and the three actions.
///
/// The card is the whole product in one unit, so everything a user needs to decide
/// sits on it: identity, the reason (plus its samples), and Follow / Not interested /
/// Copy link. The parent keys the list on the follow flag, so undoing a follow
/// remounts this card instead of leaving a stale "Following" label.
#[component]
pub(crate) fn FeedItem(item: AccountItem) -> impl IntoView {
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

    let follow_click = move || {
        if already_following {
            unfollow();
        } else {
            follow();
        }
    };

    let error_signal = error;
    // The card outlives each report, so the toast is handed down as a slot and reads the
    // shared signal for as long as it is mounted.
    let report = Signal::derive(move || {
        toast.get().map(|report| ToastReport {
            ok: report.ok,
            message: report.message,
        })
    });
    let toast_children = std::sync::Arc::new(move || view! { <Toast report=report/> }.into_any())
        as leptos::prelude::ChildrenFn;

    view! {
        <nodus_view::pages::feed::accounts::AccountCard
            display_name=display_name.clone()
            acct=acct.clone()
            avatar_url=avatar_url.clone()
            followers=followers.clone()
            reason=reason.clone()
            hint=reason_hint
            samples=samples
            note=note.clone()
            following=already_following
            busy=Signal::derive(move || busy.get())
            error=Signal::derive(move || error_signal.get())
            on_follow=Callback::new(move |()| follow_click())
            on_hide=Callback::new(move |()| not_interested())
            on_copy=Callback::new(move |()| copy())
            toast=toast_children
        />
    }
}
