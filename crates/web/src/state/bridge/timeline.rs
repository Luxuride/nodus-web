//! Reading a timeline from SDK state, and asking for it only when stale.
//!
//! The rule this module exists to enforce: a navigation is a *read*. The SDK holds
//! the pages and the statuses, so a screen paints from [`read`] before any request
//! is made, and [`ensure`] only fills what is missing or older than [`TTL_MS`].
//! Nothing here keeps a second copy — a view that cached what it rendered would be
//! the drift this whole design removes.

use nodus_instance::model::Status;
use nodus_sdk::{Timeline, TimelineRequest};

use super::handle;

/// How long a fetched page is trusted before a visit refreshes it in the background.
///
/// A minute is short enough that a reader who comes back to the tab sees new posts
/// without pressing anything, and long enough that switching between tabs is free.
pub const TTL_MS: i64 = 60_000;

/// Runs `f` with the request a `(timeline, tag)` pair names.
///
/// A closure rather than a returned value: the request borrows its tag or account id,
/// and returning it would mean allocating an owned copy on the render path.
fn with_request<R>(
    which: Timeline,
    tag: Option<&str>,
    f: impl FnOnce(TimelineRequest<'_>) -> R,
) -> R {
    match (which, tag) {
        (Timeline::Tag, Some(tag)) => f(TimelineRequest::Tag(tag)),
        (Timeline::Account(id), _) => f(TimelineRequest::Account(&id)),
        (Timeline::Local, _) => f(TimelineRequest::Local),
        (Timeline::Federated, _) => f(TimelineRequest::Federated),
        // A tag timeline without a tag is not a timeline: fall back to the home one
        // rather than panicking inside a render.
        _ => f(TimelineRequest::Home),
    }
}

/// What the SDK holds for this timeline, newest first, plus its cursor.
///
/// A pure read: no request, no clock. An empty list here means "nothing loaded yet",
/// which the caller must render as unknown rather than as "no posts".
#[must_use]
pub fn read(which: Timeline, tag: Option<&str>) -> (Vec<Status>, Option<String>) {
    let Some(sdk) = handle() else {
        return (Vec::new(), None);
    };
    with_request(which, tag, |request| sdk.read_timeline(&request))
}

/// Loads the timeline only if the SDK's copy is missing or stale.
///
/// `cursor` continues the list the reader already has; `None` asks for the newest
/// page. Returns `true` when a request was actually made, so a view can tell "I am
/// waiting on the network" from "I already had this".
pub async fn ensure(
    which: Timeline,
    tag: Option<String>,
    cursor: Option<String>,
) -> Result<bool, String> {
    let sdk = handle().ok_or_else(|| "the SDK is not installed".to_owned())?;
    let limit = super::actions::PAGE;
    let cursor = cursor.as_deref();
    // One arm per shape, because the request borrows the tag or id it names and so
    // cannot outlive this call to be awaited in a shared tail.
    let loaded = match (which, tag) {
        (Timeline::Tag, Some(tag)) => {
            sdk.ensure_timeline(TimelineRequest::Tag(&tag), cursor, limit, TTL_MS)
                .await
        }
        (Timeline::Account(id), _) => {
            sdk.ensure_timeline(TimelineRequest::Account(&id), cursor, limit, TTL_MS)
                .await
        }
        (Timeline::Local, _) => {
            sdk.ensure_timeline(TimelineRequest::Local, cursor, limit, TTL_MS)
                .await
        }
        (Timeline::Federated, _) => {
            sdk.ensure_timeline(TimelineRequest::Federated, cursor, limit, TTL_MS)
                .await
        }
        _ => {
            sdk.ensure_timeline(TimelineRequest::Home, cursor, limit, TTL_MS)
                .await
        }
    }
    .map_err(|error| error.to_string())?;
    Ok(loaded.is_some())
}

/// Stores what an action returned, so every screen showing it is corrected at once.
pub fn upsert(statuses: Vec<Status>) {
    if statuses.is_empty() {
        return;
    }
    super::dispatch(nodus_sdk::Command::StatusesUpserted { statuses });
}
