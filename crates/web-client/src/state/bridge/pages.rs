//! Reading non-timeline pages from SDK state: notifications, messages, saved, people.
//!
//! Same rule as [`super::timeline`]: a navigation is a read. A screen paints what the
//! SDK holds before any request is made, and [`ensure`] fills only what is missing or
//! older than [`TTL_MS`]. Offline, that means a page still renders — stamped with the
//! moment it was fetched, never with the pretence that it is current.

use nodus_sdk::page::{PageEntry, PageItems, PageKind};

use super::handle;

/// How long a page is trusted before a visit refreshes it.
pub const TTL_MS: i64 = 60_000;

/// What the SDK holds for this page, or `None` when nothing was loaded yet.
///
/// `None` and "an empty page" are different answers: the first means unknown, the
/// second means the server said there is nothing.
#[must_use]
pub fn read(kind: &PageKind) -> Option<PageEntry> {
    handle().and_then(|sdk| sdk.page(kind))
}

/// The items of a page, when they are of the expected shape.
#[must_use]
pub fn items<T>(
    kind: &PageKind,
    pick: impl FnOnce(PageItems) -> Option<Vec<T>>,
) -> Option<(Vec<T>, Option<String>)> {
    let entry = read(kind)?;
    let items = pick(entry.items)?;
    Some((items, entry.cursor))
}

/// Loads the newest page when the held one is missing or stale.
///
/// Returns `true` when a request was made.
pub async fn ensure(kind: PageKind) -> Result<bool, String> {
    ensure_after(kind, None).await
}

thread_local! {
    /// The pages a request is already on its way for.
    ///
    /// Two mounts of the same screen — a route alias, a tab that renders twice while the
    /// shell settles — would otherwise ask for the same page twice: the second call cannot
    /// see a fresh entry because the first has not answered yet, and the reader pays for a
    /// request whose result is thrown away. Keyed by kind and cursor, because "one more
    /// page" is a different request from "the first page".
    static IN_FLIGHT: std::cell::RefCell<std::collections::HashSet<(PageKind, Option<String>)>> =
        std::cell::RefCell::new(std::collections::HashSet::new());
}

/// Takes the in-flight slot for one page, or reports that a request already holds it.
fn claim(kind: &PageKind, cursor: Option<&str>) -> bool {
    IN_FLIGHT.with(|held| {
        held.borrow_mut()
            .insert((kind.clone(), cursor.map(ToOwned::to_owned)))
    })
}

fn release(kind: &PageKind, cursor: Option<&str>) {
    IN_FLIGHT.with(|held| {
        held.borrow_mut()
            .remove(&(kind.clone(), cursor.map(ToOwned::to_owned)));
    });
}

/// Loads one more page, older than what the reader already has.
pub async fn ensure_after(kind: PageKind, cursor: Option<String>) -> Result<bool, String> {
    if !claim(&kind, cursor.as_deref()) {
        return Ok(false);
    }
    let answer = async {
        let sdk = handle().ok_or_else(|| "the SDK is not installed".to_owned())?;
        let limit = super::actions::PAGE;
        sdk.ensure_page(kind.clone(), cursor.as_deref(), limit, TTL_MS)
            .await
            .map_err(|error| error.to_string())
    }
    .await;
    release(&kind, cursor.as_deref());
    answer
}

/// Reloads a page even when the held one is fresh.
///
/// For the editor screens: after a filter is created or a tag is featured, the local
/// copy is known to be wrong, and "known to be wrong" is the one case where the store's
/// timestamp must be ignored.
pub async fn refresh(kind: PageKind) -> Result<bool, String> {
    let sdk = handle().ok_or_else(|| "the SDK is not installed".to_owned())?;
    let limit = super::actions::PAGE;
    sdk.ensure_page(kind, None, limit, 0)
        .await
        .map_err(|error| error.to_string())
}

