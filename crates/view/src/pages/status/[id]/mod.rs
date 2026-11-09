//! The thread screen: what came before a status, the status, and the answers under it.
//!
//! The page owns the order and the containers; what one post *is* belongs to the caller,
//! which is what `card` is for. The client filters and offers verbs, the server paints the
//! shared read-only card, and neither of them can move a status out of the sequence by
//! painting it differently. Ages arrive through `age_of` for the same reason — this crate
//! reads no clock — and the composer arrives as a slot, because writing a reply is
//! transport.
//!
//! Statuses arrive as the caller prepares them: the server passes `content` already
//! sanitised and with its own custom emoji resolved (`render::status_card` does both); the
//! client passes the instance's HTML as received, because that is what the browser gets
//! anyway.

use leptos::prelude::*;
use nodus_instance::model::Status;

use crate::components::CardRenderer;
use crate::status_card::StatusCard;
use crate::Screen;

pub mod public;

/// The card a row is painted with, for as long as the page is choosing it.
type Paint = dyn Fn(Status) -> AnyView + Send + Sync;

/// One row of the thread: the caller's card when it supplied one, else the read-only card.
///
/// The age is asked for only when this function draws the header itself: a caller that
/// paints its own cards already owns a clock and a way to format a stamp with it.
fn painted_row(
    status: Status,
    card: Option<&Paint>,
    age_of: Option<Callback<String, String>>,
) -> AnyView {
    if let Some(card) = card {
        return card(status);
    }
    // A boost's age belongs to the boosted post, which is the line the card draws.
    let created_at = status
        .reblog
        .as_ref()
        .map_or(status.created_at.as_str(), |inner| {
            inner.created_at.as_str()
        })
        .to_owned();
    let age = age_of.map_or_else(String::new, |age_of| age_of.run(created_at));
    // A path is a string on both paints, so the addresses are built here rather than read
    // from a router: the server's anchors are already working links.
    let inner = status.reblog.as_deref().unwrap_or(&status);
    let author_path = format!("/accounts/{}", inner.account.id);
    let thread_path = format!("/status/{}", inner.id);
    view! {
        <StatusCard
            status=status
            age=age
            author_path=author_path
            thread_path=thread_path
            interactive=false
        />
    }
    .into_any()
}

/// One thread, from its oldest ancestor to the composer at the foot.
#[component]
pub fn ThreadPage(
    /// The statuses above the focused one, oldest first.
    ancestors: Vec<Status>,
    /// The status the address names.
    focused: Status,
    /// The answers under it, as the caller last read them.
    #[prop(into)]
    replies: Signal<Vec<Status>>,
    /// Paints one status. Absent (the server) means the shared card without verbs.
    #[prop(optional)]
    card: Option<CardRenderer>,
    /// How old a status is, in words, from the caller's clock. Only the default card reads
    /// it; a caller that paints its own cards already owns a clock.
    #[prop(optional)]
    age_of: Option<Callback<String, String>>,
    /// The reply composer, which owns the transport this crate may not have.
    #[prop(optional)]
    reply_slot: Option<ChildrenFn>,
) -> impl IntoView {
    // Everything above the answers is fixed for the life of the page, so it is painted once,
    // in reading order, before the macro: only the replies under it can grow.
    let has_ancestors = !ancestors.is_empty();
    let ancestor_rows = ancestors
        .into_iter()
        .map(|status| painted_row(status, card.as_deref(), age_of))
        .collect_view();
    let focused_row = painted_row(focused, card.as_deref(), age_of);
    // The answers repaint whenever one lands, so the renderer and the clock belong to that
    // closure: a page built once cannot lend them to it.
    let replies_card = card;
    let reply_slot = StoredValue::new(reply_slot);

    view! {
        <Screen>
            {has_ancestors.then(|| view! {
                <ul class="flex flex-col gap-3">{ancestor_rows}</ul>
            })}
            <ul class="flex flex-col gap-3">{focused_row}</ul>

            {move || {
                let answers = replies.get();
                if answers.is_empty() {
                    return ().into_any();
                }
                let count = answers.len();
                let rows = answers
                    .into_iter()
                    .map(|status| painted_row(status, replies_card.as_deref(), age_of))
                    .collect_view();
                view! {
                    <h2 class="mt-2 text-sm font-semibold opacity-70">
                        {format!("{count} replies")}
                    </h2>
                    <ul class="flex flex-col gap-3">{rows}</ul>
                }
                .into_any()
            }}

            {reply_slot.with_value(|slot| slot.as_ref().map(|slot| slot()))}
        </Screen>
    }
}
