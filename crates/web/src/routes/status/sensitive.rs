//! Whether media behind a content warning is shown without asking.
//!
//! The choice belongs to the reader and outlives the tab, so it lives in
//! `localStorage` rather than in SDK state: the server has no opinion about it, and
//! syncing it would make an accessibility preference depend on a network round trip.
//!
//! The default is to ask. A status marked sensitive is a claim by its author, and
//! opening it silently in front of whoever is looking at the screen is the one failure
//! mode this flag exists to prevent.

const KEY: &str = "nodus.sensitive";
const SHOW: &str = "show";

fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

/// Whether a reader has asked to see sensitive media without a second click.
pub fn reveals_automatically() -> bool {
    storage()
        .and_then(|storage| storage.get_item(KEY).ok().flatten())
        .is_some_and(|value| value == SHOW)
}

/// Records the reader's choice. A browser that refuses storage keeps asking, which is
/// the safe direction to fail in.
pub fn set_automatic(on: bool) {
    if let Some(storage) = storage() {
        let _ = storage.set_item(KEY, if on { SHOW } else { "blur" });
    }
}
