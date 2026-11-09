//! One notification: who did it, when, and the status it is about.
//!
//! The row is the crate's [`NotificationCard`] plus the two pieces the card cannot own,
//! because they are the two the paints disagree about: the status the instance inlined, and
//! the decision a follow request is asking for. Both arrive as a call that answers for one
//! notification, so neither paint reaches for an instance, a clock or an address to draw a
//! row.
//!
//! Without either call the row still draws both boxes — the status with the shared read-only
//! card, the request with both buttons disabled — so the two paints differ by an affordance
//! and never by a box. That is the whole rule: a reader comparing a painted page with a
//! mounted one should find the same list, one of them pressable.

use std::sync::Arc;

use leptos::prelude::*;
use nodus_instance::model::{Notification, NotificationKind, Status};

use super::request::FollowRequestActions;
use crate::NotificationCard;
use crate::status_card::{StatusCard, StatusCardProps};

/// A single notification.
#[component]
pub fn NotificationRow(
    /// The notification to draw.
    notification: Notification,
    /// How old it is, already formatted by the caller.
    age: String,
    /// Formats a timestamp as an age, for the status card this row draws itself. Absent when
    /// the caller has no clock to lend, and the card then carries no age rather than a made
    /// up one.
    #[prop(optional)]
    age_of: Option<Callback<String, String>>,
    /// Opens the thread of the status the notification is about.
    #[prop(optional)]
    on_open: Option<Callback<String>>,
    /// The client's own status card, drawn in place of the shared read-only one.
    #[prop(optional)]
    status_card: Option<Callback<Status, AnyView>>,
    /// The client's own accept/refuse controls for one follow request.
    #[prop(optional)]
    request_actions: Option<Callback<String, AnyView>>,
) -> impl IntoView {
    let is_request = notification.kind == NotificationKind::FollowRequest;
    let who = notification.account.id.clone();
    let status = notification.status.clone();

    // The request's controls sit between the heading and the status, which is where the
    // decision is read: the notification says what happened, the controls answer it.
    let extra = is_request.then(|| {
        let who = who.clone();
        Arc::new(move || match request_actions {
            Some(actions) => actions.run(who.clone()),
            None => view! { <FollowRequestActions account_id=who.clone()/> }.into_any(),
        }) as ChildrenFn
    });

    // The status the instance inlined. The client's card is the interactive one; the shared
    // card is the same boxes with the verbs it cannot own, which is what a paint with no
    // instance behind it shows.
    let body = status.map(|status| {
        Arc::new(move || {
            let card = status_card.map_or_else(
                || {
                    let status_age = age_of
                        .map_or_else(String::new, |age_of| age_of.run(status.created_at.clone()));
                    shared_card(status.clone(), status_age, on_open)
                },
                |card| card.run(status.clone()),
            );
            view! { <ul class="flex flex-col gap-2">{card}</ul> }.into_any()
        }) as ChildrenFn
    });

    view! {
        <NotificationCard notification=notification age=age extra=extra body=body/>
    }
}

/// The shared card for a status the instance inlined, with the seams this row forwards.
///
/// The props are built as a struct rather than through `view!` because `on_open` is optional:
/// its setter takes the bare callback, and a row that only has the option to forward would
/// have to invent one.
fn shared_card(status: Status, age: String, on_open: Option<Callback<String>>) -> AnyView {
    let author_path = format!("/accounts/{}", status.account.id);
    let thread_path = format!("/status/{}", status.id);

    StatusCard(StatusCardProps {
        status,
        age,
        author_path,
        thread_path,
        on_navigate: None,
        on_open,
        on_boost: None,
        on_favourite: None,
        on_bookmark: None,
        on_reply: None,
        before_verbs: None,
        after_verbs: None,
        on_open_media: None,
        on_vote: None,
        interactive: false,
        body: None,
        extra: None,
    })
    .into_any()
}
