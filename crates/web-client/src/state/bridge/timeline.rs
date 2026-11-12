//! Reading a timeline from SDK state, and asking for it only when stale.
//!
//! The rule this module exists to enforce: a navigation is a *read*. The SDK holds
//! the pages and the statuses, so a screen paints from [`read`] before any request
//! is made, and [`ensure`] only fills what is missing or older than [`TTL_MS`].
//! Nothing here keeps a second copy — a view that cached what it rendered would be
//! the drift this whole design removes.

use nodus_client_core::model::Status;
use nodus_client_sdk::{Timeline, TimelineRequest};

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
    ensure_with(which, tag, cursor, TTL_MS).await
}

/// Whether the store holds this timeline at all, fresh or stale.
#[must_use]
pub fn held(which: Timeline, tag: Option<&str>) -> bool {
    let Some(sdk) = handle() else {
        return false;
    };
    with_request(which, tag, |request| {
        sdk.timeline_fetched_at(&request).is_some()
    })
}

/// Loads the newest page even when the held one is still fresh.
///
/// Used after the reader posts: their own status is older than nothing, but the
/// held page was fetched before it existed, so trusting its timestamp would show a
/// timeline missing the post the reader just wrote.
pub async fn refresh(which: Timeline, tag: Option<String>) -> Result<bool, String> {
    ensure_with(which, tag, None, 0).await
}

thread_local! {
    /// The timelines a request is already on its way for.
    ///
    /// The alias `/home` redirects to `/`, and the redirect mounts the screen a second time
    /// before the first answer arrives, so both ask for the same page and the reader pays
    /// twice. Keyed by what the request names, not by the request's parameters: the same
    /// timeline at a different cursor is a different page.
    static IN_FLIGHT: std::cell::RefCell<std::collections::HashSet<(String, Option<String>)>> =
        std::cell::RefCell::new(std::collections::HashSet::new());
}

/// A key that names this timeline and page, or `None` when the shape is malformed.
fn claim_key(
    which: &Timeline,
    tag: Option<&str>,
    cursor: Option<&str>,
) -> (String, Option<String>) {
    let name = match (which, tag) {
        (Timeline::Tag, Some(tag)) => format!("tag:{tag}"),
        (Timeline::Account(id), _) => format!("account:{id}"),
        (Timeline::List(id), _) => format!("list:{id}"),
        (Timeline::Tag | Timeline::Home | Timeline::Local | Timeline::Federated, _) => {
            format!("{which:?}")
        }
    };
    (name, cursor.map(ToOwned::to_owned))
}

/// Whether this call took the in-flight slot, or a request already holds it.
fn claim(key: &(String, Option<String>)) -> bool {
    IN_FLIGHT.with(|held| held.borrow_mut().insert(key.clone()))
}

fn release(key: &(String, Option<String>)) {
    IN_FLIGHT.with(|held| {
        held.borrow_mut().remove(key);
    });
}

async fn ensure_with(
    which: Timeline,
    tag: Option<String>,
    cursor: Option<String>,
    ttl_ms: i64,
) -> Result<bool, String> {
    let key = claim_key(&which, tag.as_deref(), cursor.as_deref());
    if !claim(&key) {
        return Ok(false);
    }
    let answer = ensure_requested(which, tag, cursor, ttl_ms).await;
    release(&key);
    answer
}

/// The one implementation: `ttl_ms` of zero means "ask".
async fn ensure_requested(
    which: Timeline,
    tag: Option<String>,
    cursor: Option<String>,
    ttl_ms: i64,
) -> Result<bool, String> {
    let sdk = handle().ok_or_else(|| "the SDK is not installed".to_owned())?;
    let limit = super::actions::PAGE;
    let cursor = cursor.as_deref();
    // One arm per shape, because the request borrows the tag or id it names and so
    // cannot outlive this call to be awaited in a shared tail.
    let loaded = match (which, tag) {
        (Timeline::Tag, Some(tag)) => {
            sdk.ensure_timeline(TimelineRequest::Tag(&tag), cursor, limit, ttl_ms)
                .await
        }
        (Timeline::Account(id), _) => {
            sdk.ensure_timeline(TimelineRequest::Account(&id), cursor, limit, ttl_ms)
                .await
        }
        (Timeline::List(id), _) => {
            // The list loader is the one path `ensure_timeline` cannot cover (its arm
            // would fall through to the home stream), so the freshness question is
            // answered here — otherwise every visit to `/lists` refetched what the
            // server had just painted.
            if cursor.is_none() {
                let now = js_sys::Date::now() as i64;
                let fresh = sdk
                    .timeline_fetched_at(&TimelineRequest::List(&id))
                    .is_some_and(|at| now.saturating_sub(at) < ttl_ms);
                if fresh {
                    return Ok(false);
                }
            }
            // Lists have their own loader: reading one needs the `Browse` port, which the
            // SDK keeps out of the bound every caller of `ensure_timeline` would pay.
            return sdk
                .load_list_page(&id, cursor, limit)
                .await
                .map(|page| page.fetched_at.is_some())
                .map_err(|error| error.to_string());
        }
        (Timeline::Local, _) => {
            sdk.ensure_timeline(TimelineRequest::Local, cursor, limit, ttl_ms)
                .await
        }
        (Timeline::Federated, _) => {
            sdk.ensure_timeline(TimelineRequest::Federated, cursor, limit, ttl_ms)
                .await
        }
        _ => {
            sdk.ensure_timeline(TimelineRequest::Home, cursor, limit, ttl_ms)
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
    super::dispatch(nodus_client_sdk::Command::StatusesUpserted { statuses });
}
