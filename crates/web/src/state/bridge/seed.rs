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

use super::with_sdk;

static ADOPTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// What the server writes into `#nodus-page`.
#[derive(Debug, serde::Deserialize)]
struct Island {
    which: String,
    #[serde(default)]
    statuses: Vec<Status>,
    #[serde(default)]
    cursor: Option<String>,
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
    if island.which != "home" || island.statuses.is_empty() {
        return;
    }
    with_sdk(|sdk| {
        sdk.adopt_page(
            Timeline::Home,
            None,
            island.statuses.clone(),
            island.cursor.clone(),
        );
    });
}
