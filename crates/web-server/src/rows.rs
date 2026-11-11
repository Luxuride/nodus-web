//! One row of a list, and the status card that sits inside a notification.
//!
//! Every function here renders a component from `nodus-view` and returns the string the page
//! embeds: the markup is the client's markup, this module supplies only the data and the
//! absent verbs.

use leptos::prelude::*;
use nodus_instance::model::Status;

/// One status as markup: who, what, and what can be done with it.
///
/// The whole card comes from `nodus-view`, the component the client mounts, so the first
/// paint and the one after the wasm boots cannot drift. The body is rewritten here rather
/// than in the component: the emoji shortcodes are a read of the instance's emoji list, and
/// `inner_html` already carries the HTML both paints show.
pub fn status_card(status: &Status) -> String {
    let inner = status.reblog.as_deref().unwrap_or(status);
    painted(
        status,
        &format!("/accounts/{}", inner.account.id),
        &format!("/status/{}", inner.id),
    )
}

/// The same card for a visitor: its links stay on the instance that hosts the post.
pub fn status_card_public(status: &Status, host: &str) -> String {
    let inner = status.reblog.as_deref().unwrap_or(status);
    painted(
        status,
        &format!("/accounts/{host}/{}", inner.account.id),
        &format!("/status/{host}/{}", inner.id),
    )
}

fn painted(status: &Status, author_path: &str, thread_path: &str) -> String {
    let mut painted = status.clone();
    // A boost renders the boosted post under the booster's name, exactly as the client
    // does: the line above the card comes from the wrapper and everything below it from
    // `reblog`.
    match painted.reblog.as_deref_mut() {
        Some(inner) => inner.content = crate::content::prepared(&inner.content, &inner.emojis),
        None => painted.content = crate::content::prepared(&painted.content, &painted.emojis),
    }
    let inner = painted.reblog.as_deref().map_or(&painted, |inner| inner);
    let age = nodus_ui::relative_age(&inner.created_at, now_millis());

    Owner::new().with(|| {
        nodus_ui::status_card::StatusCard(nodus_ui::status_card::StatusCardProps {
            status: painted,
            age,
            author_path: author_path.to_owned(),
            thread_path: thread_path.to_owned(),
            on_navigate: Some(nodus_route::route::noop()),
            on_open: Some(nodus_route::route::noop()),
            on_boost: Some(nodus_route::route::noop()),
            on_favourite: Some(nodus_route::route::noop()),
            on_bookmark: Some(nodus_route::route::noop()),
            on_reply: Some(nodus_route::route::noop()),
            before_verbs: None,
            after_verbs: None,
            on_open_media: Some(nodus_route::route::noop()),
            on_vote: Some(nodus_route::route::noop()),
            interactive: true,
            body: None,
            extra: None,
        })
        .to_html()
    })
}

/// Milliseconds since the Unix epoch, by this host's clock.
pub(crate) fn now_millis() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0.0, |elapsed| elapsed.as_secs_f64() * 1_000.0)
}
