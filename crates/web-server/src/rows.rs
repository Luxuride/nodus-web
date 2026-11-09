//! One row of a list, and the status card that sits inside a notification.
//!
//! Every function here renders a component from `nodus-view` and returns the string the page
//! embeds: the markup is the client's markup, this module supplies only the data and the
//! absent verbs.

use leptos::prelude::*;
use nodus_instance::model::{Account, Conversation, List, Notification, ScheduledStatus, Status};

use crate::document::escape;
use crate::html::body as body_html;

/// One status as markup: who, what, and what can be done with it.
///
/// The whole card comes from `nodus-view`, the component the client mounts, so the first
/// paint and the one after the wasm boots cannot drift. The body is rewritten here rather
/// than in the component: the emoji shortcodes are a read of the instance's emoji list, and
/// `inner_html` already carries the HTML both paints show.
pub fn status_card(status: &Status) -> String {
    let mut painted = status.clone();
    // A boost renders the boosted post under the booster's name, exactly as the client
    // does: the line above the card comes from the wrapper and everything below it from
    // `reblog`.
    match painted.reblog.as_deref_mut() {
        Some(inner) => inner.content = crate::content::prepared(&inner.content, &inner.emojis),
        None => painted.content = crate::content::prepared(&painted.content, &painted.emojis),
    }
    let inner = painted.reblog.as_deref().map_or(&painted, |inner| inner);
    let age = nodus_view::relative_age(&inner.created_at, now_millis());
    let author_path = format!("/accounts/{}", inner.account.id);
    let thread_path = format!("/status/{}", inner.id);

    Owner::new().with(|| {
        nodus_view::status_card::StatusCard(nodus_view::status_card::StatusCardProps {
            status: painted,
            age,
            author_path,
            thread_path,
            on_navigate: None,
            on_open: None,
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
        .to_html()
    })
}

/// A list of rows, or the sentence that says the list is empty.
pub fn rows(items: &[String], empty: &str) -> String {
    // The classes come from `nodus_view` so a change lands on both paints at once; the
    // fragment itself is assembled here because the rows are already-rendered components and
    // a `ChildrenFn` cannot be built from a list of strings.
    if items.is_empty() {
        return format!(
            "<p class=\"{}\">{}</p>",
            nodus_view::lists::rows::EMPTY,
            escape(empty)
        );
    }
    format!(
        "<ul class=\"{}\">{}</ul>",
        nodus_view::lists::rows::STACK,
        items.concat()
    )
}

pub fn account_row(account: &Account) -> String {
    let note = body_html(&account.note);
    Owner::new().with(|| {
        nodus_view::lists::account_row::AccountRow(
            nodus_view::lists::account_row::AccountRowProps {
                account: account.clone(),
                note,
                on_open: None,
                after: None,
            },
        )
        .to_html()
    })
}

pub fn notification_row(notification: &Notification) -> String {
    use leptos::prelude::*;

    let age = nodus_view::relative_age(&notification.created_at, now_millis());
    let body = notification.status.as_ref().map(status_card).map(|html| {
        let body = std::sync::Arc::new(move || {
            leptos::html::ul()
                .class("flex flex-col gap-2")
                .inner_html(html.clone())
                .into_any()
        });
        body as ChildrenFn
    });
    let notification = notification.clone();

    Owner::new().with(|| {
        nodus_view::NotificationCard(nodus_view::NotificationCardProps {
            notification,
            age,
            body,
            extra: None,
        })
        .to_html()
    })
}

pub fn conversation_row(conversation: &Conversation) -> String {
    let age = conversation
        .last_status
        .as_ref()
        .map_or_else(String::new, |status| {
            nodus_view::relative_age(&status.created_at, now_millis())
        });
    let card = conversation
        .last_status
        .as_ref()
        .map(status_card)
        .unwrap_or_default();
    let last: Option<std::sync::Arc<dyn Fn() -> leptos::prelude::AnyView + Send + Sync>> =
        Some(std::sync::Arc::new(move || {
            // `Fn`, not `FnOnce`: the closure may be called more than once.
            leptos::html::div().inner_html(card.clone()).into_any()
        }));
    Owner::new().with(|| {
        nodus_view::lists::conversation_row::ConversationRow(
            nodus_view::lists::conversation_row::ConversationRowProps {
                conversation: conversation.clone(),
                age,
                // The server cannot change a conversation, so it draws the row without the
                // verbs and without a mute state it would have to guess.
                muted: false,
                on_open: None,
                on_read: None,
                status: last,
                after: None,
            },
        )
        .to_html()
    })
}

pub fn list_row(list: &List) -> String {
    Owner::new().with(|| {
        nodus_view::lists::list_row::ListRow(nodus_view::lists::list_row::ListRowProps {
            list: list.clone(),
            selected: false,
            on_select: None,
            children: None,
        })
        .to_html()
    })
}

pub fn scheduled_row(scheduled: &ScheduledStatus) -> String {
    let age = nodus_view::relative_age(&scheduled.scheduled_at, now_millis());
    Owner::new().with(|| {
        nodus_view::lists::scheduled_row::ScheduledRow(
            nodus_view::lists::scheduled_row::ScheduledRowProps {
                scheduled: scheduled.clone(),
                age,
                on_cancel: None,
            },
        )
        .to_html()
    })
}

/// Milliseconds since the Unix epoch, by this host's clock.
pub(crate) fn now_millis() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0.0, |elapsed| elapsed.as_secs_f64() * 1_000.0)
}
