//! User actions against the instance: the verbs that are not state.
//!
//! The SDK deliberately owns *state* — session, cursors, the feed snapshot — and not
//! these. A favourite is a thing the user did, and what it returns is the instance's own
//! answer, so the view calls one of these, keeps the returned status, and re-renders from
//! it. Nothing here writes into `AppState`: an optimistic toggle that the server later
//! refused would be a lie the state machine cannot retract.

use core::future::Future;

use nodus_instance::BrowserInstance;
use nodus_instance::InstanceApi;
use nodus_instance::actions::{AccountActions, Browse, Search, StatusActions};
use nodus_instance::model::{
    Account, Context, Draft, HistoryEntry, LocalId, Notification, Page, PageParams, Relationship,
    SearchResults, Status,
};

pub use nodus_instance::model::Visibility;

use super::instance_transport;

/// The transport for the current session, or a message fit for inline display.
fn transport() -> Result<BrowserInstance, String> {
    instance_transport().ok_or_else(|| "Not signed in.".to_owned())
}

/// Runs one transport call, flattening its error into a display string.
async fn call<T>(
    future: impl Future<Output = Result<T, nodus_instance::Error>>,
) -> Result<T, String> {
    future.await.map_err(|error| error.to_string())
}

/// The page size every list view asks for.
///
/// One constant, because a mismatch between "what was loaded" and "what the cursor
/// assumed" is how a timeline skips or repeats an entry.
pub const PAGE: u16 = 20;

/// Pages a home timeline.
pub async fn home_page(after: Option<&str>) -> Result<Page<Status>, String> {
    let transport = transport()?;
    let params = match after {
        Some(cursor) => PageParams::after(cursor, PAGE),
        None => PageParams::first(PAGE),
    };
    call(transport.timeline(nodus_instance::model::StreamKind::User, params)).await
}

/// Pages a public timeline; `local` keeps it to this instance.
pub async fn public_page(after: Option<&str>, local: bool) -> Result<Page<Status>, String> {
    let transport = transport()?;
    let params = match after {
        Some(cursor) => PageParams::after(cursor, PAGE),
        None => PageParams::first(PAGE),
    };
    call(transport.timeline(
        if local {
            nodus_instance::model::StreamKind::Local
        } else {
            nodus_instance::model::StreamKind::Federated
        },
        params,
    ))
    .await
}

/// One status.
pub async fn status(id: &str) -> Result<Status, String> {
    call(transport()?.status(id)).await
}

/// The ancestors and descendants of a status.
pub async fn thread(id: &str) -> Result<Context, String> {
    call(transport()?.status_context(id)).await
}

/// One account.
pub async fn account(id: &str) -> Result<Account, String> {
    let id = LocalId(id.to_owned());
    call(transport()?.account(&id)).await
}

/// An account's statuses, newest first.
pub async fn account_statuses(id: &str, after: Option<&str>) -> Result<Page<Status>, String> {
    let id = LocalId(id.to_owned());
    let params = match after {
        Some(cursor) => PageParams::after(cursor, PAGE),
        None => PageParams::first(PAGE),
    };
    call(transport()?.account_statuses(&id, params)).await
}

/// The relationship between the signed-in account and `id`.
pub async fn relationship(id: &str) -> Result<Option<Relationship>, String> {
    let ids = [LocalId(id.to_owned())];
    let found = call(transport()?.relationships(&ids)).await?;
    Ok(found.into_iter().next())
}

/// The account's notifications, newest first.
pub async fn notifications(after: Option<&str>) -> Result<Page<Notification>, String> {
    let params = match after {
        Some(cursor) => PageParams::after(cursor, PAGE),
        None => PageParams::first(PAGE),
    };
    call(transport()?.notifications(params)).await
}

/// Favourites or un-favourites a status.
pub async fn favourite(id: &str, on: bool) -> Result<Status, String> {
    let transport = transport()?;
    if on {
        call(transport.favourite(id)).await
    } else {
        call(transport.unfavourite(id)).await
    }
}

/// Boosts or un-boosts a status.
pub async fn reblog(id: &str, on: bool) -> Result<Status, String> {
    let transport = transport()?;
    if on {
        call(transport.reblog(id)).await
    } else {
        call(transport.unreblog(id)).await
    }
}

/// Bookmarks or un-bookmarks a status.
pub async fn bookmark(id: &str, on: bool) -> Result<Status, String> {
    let transport = transport()?;
    if on {
        call(transport.bookmark(id)).await
    } else {
        call(transport.unbookmark(id)).await
    }
}

/// Deletes a status the signed-in account wrote.
#[expect(
    dead_code,
    reason = "the delete button lands with the thread view's own-status menu"
)]
pub async fn delete_status(id: &str) -> Result<(), String> {
    call(transport()?.delete_status(id)).await
}

/// Publishes a status.
pub async fn publish(
    text: &str,
    visibility: Visibility,
    reply_to: Option<&str>,
) -> Result<Status, String> {
    let transport = transport()?;
    let draft = Draft {
        text: text.to_owned(),
        visibility,
        in_reply_to_id: reply_to.map(str::to_owned),
        // A new key per attempt: the point is that a retried *request* cannot post
        // twice, not that the same text may never be posted twice.
        idempotency_key: Some(format!("nodus-{}", js_sys::Date::now())),
    };
    call(transport.post_status(&draft)).await
}

/// Votes on a poll and returns the status it belongs to.
pub async fn vote(poll_id: &str, choices: &[usize]) -> Result<Status, String> {
    call(transport()?.vote(poll_id, choices)).await
}

/// The edit history of a status, oldest first.
#[expect(dead_code, reason = "the edit-history panel lands with editing")]
pub async fn history(id: &str) -> Result<Vec<HistoryEntry>, String> {
    call(transport()?.history(id)).await
}

/// Follows or unfollows an account.
pub async fn follow(id: &str, on: bool) -> Result<Relationship, String> {
    let id = LocalId(id.to_owned());
    let transport = transport()?;
    // Follow lives on `InstanceApi` because the recommender's sync worker uses it, so
    // it needs no action trait; the branch is here because the two futures differ.
    if on {
        call(transport.follow(&id)).await
    } else {
        call(transport.unfollow(&id)).await
    }
}

/// Blocks or unblocks an account.
pub async fn block(id: &str, on: bool) -> Result<Relationship, String> {
    let id = LocalId(id.to_owned());
    let transport = transport()?;
    if on {
        call(transport.block(&id)).await
    } else {
        call(transport.unblock(&id)).await
    }
}

/// Mutes or unmutes an account.
pub async fn mute(id: &str, on: bool) -> Result<Relationship, String> {
    let id = LocalId(id.to_owned());
    let transport = transport()?;
    if on {
        call(transport.mute(&id)).await
    } else {
        call(transport.unmute(&id)).await
    }
}

/// Searches accounts, statuses and hashtags; `resolve` lets the instance fetch remote
/// matches it has never seen.
pub async fn search(query: &str, resolve: bool) -> Result<SearchResults, String> {
    call(transport()?.search(query, resolve)).await
}

/// The statuses the signed-in account favourited.
pub async fn favourites(after: Option<&str>) -> Result<Page<Status>, String> {
    let params = match after {
        Some(cursor) => PageParams::after(cursor, PAGE),
        None => PageParams::first(PAGE),
    };
    call(transport()?.favourites(params)).await
}

/// The statuses the signed-in account bookmarked.
pub async fn bookmarks(after: Option<&str>) -> Result<Page<Status>, String> {
    let params = match after {
        Some(cursor) => PageParams::after(cursor, PAGE),
        None => PageParams::first(PAGE),
    };
    call(transport()?.bookmarks(params)).await
}
