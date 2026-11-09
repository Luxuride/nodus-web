//! Reporting what the client was shown, and for how long.
//!
//! An impression is the denominator of every other signal: the server shrinks a
//! rejection by `n / (n + 3)` impressions and counts `seen_count_7d` from them, so
//! without impressions a penalty never decays and fatigue never registers.
//!
//! The row is written **once per item, when the card leaves the screen** (D36). The
//! alternative — reporting on arrival — was cheaper but could never carry dwell, and
//! reporting on both paths would double the denominator, because the server
//! deduplicates on `(item_id, kind, client_ts)` and two sends have two timestamps.
//!
//! Between the two events this module only keeps a map, which means the accounting
//! lives in `nodus_sdk::DwellTracker` (tested natively) and the browser work here is
//! reading the clock and moving bytes.

use std::cell::RefCell;

use nodus_sdk::{DwellTracker, MAX_EVENTS_PER_BATCH};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;

use super::handle;

/// Flush once this many impressions are queued, so a long scroll does not wait for
/// the debounce timer.
const FLUSH_EVERY: usize = 20;

/// How long a partial batch waits before it is sent.
///
/// Short enough that a session's impressions land while the page is still open, long
/// enough that scrolling through twenty cards does not produce twenty requests.
const FLUSH_DELAY_MS: i32 = 2_000;

thread_local! {
    static TRACKER: RefCell<DwellTracker> = RefCell::new(DwellTracker::new());
    /// The pending debounce timer, kept alive so the browser does not collect it.
    static TIMER: RefCell<Option<Closure<dyn FnMut()>>> = const { RefCell::new(None) };
}

/// Milliseconds from the browser clock.
///
/// The only loss is sub-millisecond precision, which no dwell measurement cares
/// about; the cast is the whole point of the function.
#[expect(
    clippy::cast_possible_truncation,
    reason = "milliseconds since the epoch are integral in practice and fit i64"
)]
fn now_ms() -> i64 {
    js_sys::Date::now() as i64
}

/// RFC 3339 timestamp from the browser clock.
fn client_ts() -> String {
    js_sys::Date::new_0().to_iso_string().into()
}

/// Names the list the next cards belong to, flushing anything already queued.
///
/// Called when a feed page is remembered. The flush first, because the queued events
/// name the previous list and their envelope may not be reused for the new one.
pub fn set_feed(feed_id: &str) {
    flush();
    TRACKER.with(|tracker| tracker.borrow_mut().set_feed(feed_id));
}

/// Records that a card is on screen.
///
/// Safe to call on every render: the tracker keeps the earliest start for an item, so
/// a re-render cannot restart the clock.
pub fn begin(item_id: i64) {
    TRACKER.with(|tracker| tracker.borrow_mut().begin(item_id, now_ms()));
}

/// Records that a card left the screen and queues its impression.
pub fn finish(item_id: i64) {
    let queued = TRACKER.with(|tracker| {
        let mut tracker = tracker.borrow_mut();
        tracker.finish(item_id, now_ms(), &client_ts())
    });
    if queued {
        schedule_flush();
    }
}

/// Ends every card on screen — used when the page is being hidden.
pub fn finish_all() {
    TRACKER.with(|tracker| {
        let mut tracker = tracker.borrow_mut();
        let _ = tracker.finish_all(now_ms(), &client_ts());
    });
    flush();
}

/// Drops everything tracked, without reporting it.
///
/// For sign-out: the rows would name a user who no longer exists, and an impression
/// for them is a count against nobody.
pub fn forget() {
    TRACKER.with(|tracker| tracker.borrow_mut().discard());
}

/// Installs the page-hide flush.
///
/// A hidden page is the last moment the browser lets us work: cards still on screen
/// have to be reported there or not at all, which is the reporting hole this design
/// accepts and documents rather than hides.
pub fn install() {
    let Some(window) = web_sys::window() else {
        return;
    };
    let handler = Closure::<dyn FnMut(web_sys::Event)>::new(|_event: web_sys::Event| {
        finish_all();
    });
    let _ = window.add_event_listener_with_callback("pagehide", handler.as_ref().unchecked_ref());
    // The listener lives as long as the page does; dropping the closure would leave
    // the browser calling into freed memory.
    handler.forget();
}

/// Sends everything queued, grouped into batches the server accepts.
pub fn flush() {
    let batches = TRACKER.with(|tracker| tracker.borrow_mut().take_batches(MAX_EVENTS_PER_BATCH));
    if batches.is_empty() {
        return;
    }
    let Some(sdk) = handle() else {
        return;
    };
    for batch in batches {
        let sdk = sdk.clone();
        wasm_bindgen_futures::spawn_local(async move {
            // Fire and forget: an impression is not a user action, so a failed batch
            // is dropped rather than retried. Missing observations make a counter
            // noisier; duplicates would make it wrong.
            let _ = sdk.report_events(batch).await;
        });
    }
}

/// Flushes now when the queue is long, otherwise schedules a delayed flush.
fn schedule_flush() {
    let pending = TRACKER.with(|tracker| tracker.borrow().pending());
    if pending >= FLUSH_EVERY {
        flush();
        return;
    }

    TIMER.with(|slot| {
        if slot.borrow().is_some() {
            // A timer is already running and will pick the new event up.
            return;
        }
        let callback = Closure::<dyn FnMut()>::once(move || {
            TIMER.with(|slot| *slot.borrow_mut() = None);
            flush();
        });
        if let Some(window) = web_sys::window() {
            let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                callback.as_ref().unchecked_ref(),
                FLUSH_DELAY_MS,
            );
        }
        // Stored after the call so the timer cannot run while the slot is borrowed.
        TIMER.with(|slot| *slot.borrow_mut() = Some(callback));
    });
}
