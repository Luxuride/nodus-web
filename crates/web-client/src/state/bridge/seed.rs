//! Adopt the page the SSR host already painted.
//!
//! The server fetches the first page of the home timeline, paints it, and writes the same
//! statuses into a JSON island. Reading that island before the app mounts is what removes
//! the last spinner: by the time any component renders, the store already holds the page,
//! so `timeline::held` is true and `ensure` answers without a request.
//!
//! Nothing here is required for the app to work — a bundle served without the island (the
//! static deployment, a reload of a client-side route) simply starts cold, as before.

use nodus_instance::model::Status;
use nodus_sdk::Timeline;
use nodus_sdk::page::{PageItems, PageKind};

use super::{handle, with_sdk};

static ADOPTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

thread_local! {
    /// An island read before the SDK existed. `main` reads the island first (the mount
    /// replaces the node it lives in), but the SDK is installed during `bridge::init`,
    /// which runs later — so the parsed page waits here rather than panicking.
    static PENDING: std::cell::RefCell<Option<Island>> = const { std::cell::RefCell::new(None) };
}

/// What the server writes into `#nodus-page`.
#[derive(Debug, serde::Deserialize)]
struct Island {
    /// A timeline ("home", "local", "federated") or a page kind ("notifications", …).
    #[serde(default)]
    which: String,
    #[serde(default)]
    kind: String,
    #[serde(default)]
    statuses: Vec<Status>,
    #[serde(default)]
    items: Option<serde_json::Value>,
    #[serde(default)]
    cursor: Option<String>,
}

/// The page the island carries, when it is one of the lists a screen reads from the store.
///
/// The server paints these the same way it paints a timeline, so a reader who arrives on
/// `/notifications` should not watch the client fetch the page they are already looking at.
fn page_of(island: &Island) -> Option<(PageKind, PageItems)> {
    let values = island.items.as_ref()?.as_array()?.clone();
    Some(match island.kind.as_str() {
        "notifications" => (
            PageKind::Notifications,
            PageItems::Notifications(parse_typed(&values)),
        ),
        "conversations" => (
            PageKind::Conversations,
            PageItems::Conversations(parse_typed(&values)),
        ),
        "favourites" => (
            PageKind::Favourites,
            PageItems::Statuses(parse_typed(&values)),
        ),
        "bookmarks" => (
            PageKind::Bookmarks,
            PageItems::Statuses(parse_typed(&values)),
        ),
        "scheduled" => (
            PageKind::Scheduled,
            PageItems::Scheduled(parse_typed(&values)),
        ),
        "directory" => (
            PageKind::Directory(true),
            PageItems::Accounts(parse_typed(&values)),
        ),
        _ => return None,
    })
}

/// Deserialise a JSON array into a typed list, dropping what does not fit.
///
/// A page is worth adopting even when one item changed shape between the server's build and
/// this one; an island that cannot be read is a cold start, not a broken screen.
fn parse_typed<T: serde::de::DeserializeOwned>(items: &[serde_json::Value]) -> Vec<T> {
    items
        .iter()
        .filter_map(|item| serde_json::from_value(item.clone()).ok())
        .collect()
}

/// Read the island, if this document has one, and put its page in the store.
///
/// Called from `main` before the mount, because the island lives inside the node the mount
/// replaces. Idempotent: the island can only be adopted once.
pub fn install() {
    if ADOPTED.swap(true, std::sync::atomic::Ordering::Relaxed) {
        return;
    }
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };
    let Some(element) = document.get_element_by_id("nodus-page") else {
        return;
    };
    let Some(text) = element.text_content() else {
        return;
    };
    let Ok(island) = serde_json::from_str::<Island>(&text) else {
        // A malformed island is a server bug, and the app still works without it: the
        // client fetches the page it could not adopt.
        web_sys::console::warn_1(&"nodus: could not read the server-painted page".into());
        return;
    };
    let usable = if island.kind.is_empty() {
        !island.statuses.is_empty() && timeline_for(&island.which).is_some()
    } else {
        page_of(&island).is_some()
    };
    if !usable {
        return;
    }
    if handle().is_some() {
        adopt(island);
    } else {
        PENDING.with(|pending| *pending.borrow_mut() = Some(island));
    }
}

/// Adopts an island that was read before the SDK was installed.
pub fn adopt_pending() {
    let pending = PENDING.with(|pending| pending.borrow_mut().take());
    if let Some(island) = pending {
        adopt(island);
    }
}

/// The store's name for the page the server painted.
///
/// Only the timelines share one island shape; the other screens carry their own lists and
/// are still fetched after the mount. `None` means "this island is not one I can adopt".
fn timeline_for(which: &str) -> Option<Timeline> {
    match which {
        "home" => Some(Timeline::Home),
        "local" => Some(Timeline::Local),
        "federated" => Some(Timeline::Federated),
        _ => None,
    }
}

fn adopt(island: Island) {
    if !island.kind.is_empty() {
        if let Some((kind, items)) = page_of(&island) {
            with_sdk(|sdk| sdk.adopt_items(kind, items, island.cursor));
        }
        return;
    }
    let Some(which) = timeline_for(&island.which) else {
        return;
    };
    with_sdk(|sdk| {
        sdk.adopt_page(which, None, island.statuses, island.cursor);
    });
}
