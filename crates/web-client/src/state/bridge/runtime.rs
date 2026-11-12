//! The background tick, in the browser.
//!
//! The SDK decides *what* work is due (`Driver::next_wake`), and this module decides
//! *when* to ask. In the browser that difference matters: there is no thread to
//! park, timers are throttled in hidden tabs (roughly one per minute), and a sleeping
//! laptop can skip hours at a time. So the loop never assumes it woke on time — it
//! asks the driver again after every sleep, and a missed deadline simply fires late.
//!
//! The loop is also deliberately quiet: a background refresh nobody asked for must
//! not raise a toast or a spinner. Only the views' own requests show progress; the
//! tick's failures are folded into the driver's backoff (see `Sdk::drive`).

use std::cell::RefCell;
use std::time::Duration;

use web_time::Instant;

use leptos::task::spawn_local;
use nodus_client_sdk::Driver;

use super::SDK;

/// Longest the loop sleeps when the driver has no opinion.
///
/// Without a deadline the only useful thing to do is check again later: a tab can be
/// hidden for a long time, and a minute costs nothing because nothing runs.
const IDLE_SLEEP: Duration = Duration::from_secs(60);

/// Shortest time the loop will ever wait.
///
/// `Driver::next_wake` answers `Some(0)` when work is due now, which is the right
/// answer for a caller that owns a thread. In a browser it is a trap: a due plan that
/// the SDK declines to act on (signed out, or a cycle that records no attempt) stays
/// due, and a zero-delay loop starves the main thread — the page keeps rendering
/// nothing, so clicks appear to do nothing at all. One second of guaranteed yielding
/// bounds that at one tick a second and keeps painting.
const MIN_TICK: Duration = Duration::from_secs(1);

thread_local! {
    /// The driver's budget, kept out of the SDK because it describes the *process*.
    static DRIVER: RefCell<Option<Driver>> = const { RefCell::new(None) };
}

/// Starts the loop. Idempotent, so a remount does not start a second one.
pub fn start() {
    if DRIVER.with_borrow(Option::is_some) {
        return;
    }
    let mut driver = Driver::new();
    driver.record_success(Instant::now());
    DRIVER.with_borrow_mut(|slot| *slot = Some(driver));

    spawn_local(async move {
        // Counts ticks that were due the moment they started. A run of them means the
        // plan is stuck on a flag the SDK cannot clear; the floor below keeps the page
        // alive and this warning is what makes the situation visible in a console.
        let mut immediate = 0_u32;
        let mut warned = false;
        loop {
            let due = DRIVER.with_borrow(|slot| {
                slot.as_ref()
                    .and_then(|driver| driver.next_wake(Instant::now()))
            });

            let sleep_for = due.map_or(IDLE_SLEEP, |wait| wait.min(IDLE_SLEEP));
            if !sleep_for.is_zero() {
                immediate = 0;
                sleep_millis(sleep_for.max(MIN_TICK)).await;
                continue;
            }

            immediate = immediate.saturating_add(1);
            if immediate >= 10 && !warned {
                warned = true;
                web_sys::console::warn_1(
                    &"nodus: the background tick is due on every wake without making progress"
                        .into(),
                );
            }

            // Due. Take the driver out so the SDK can borrow it mutably while the
            // state handles are borrowed elsewhere; put it back before awaiting
            // anything else.
            let Some(mut driver) = DRIVER.with_borrow_mut(Option::take) else {
                return;
            };
            let Some(sdk) = SDK.with_borrow(std::clone::Clone::clone) else {
                // Not installed yet, so this tick cannot act. Put the driver back and
                // wait rather than returning: a loop that exits here would never
                // restart, and the whole point of the tick is that it outlives a mount.
                DRIVER.with_borrow_mut(|slot| *slot = Some(driver));
                sleep_millis(MIN_TICK).await;
                continue;
            };

            // No session, nothing to refresh: the driver keeps its budget and the
            // loop waits for the next wakeup, which `sign_in` will bring forward.
            if sdk.state().session.get().is_none() {
                driver.record_success(Instant::now());
            } else {
                let now = Instant::now();
                let _ = sdk.drive(&mut driver, now).await;
            }

            DRIVER.with_borrow_mut(|slot| *slot = Some(driver));
            // Always yield after acting: the next plan may still be due, and the frame
            // has to be painted before another tick is allowed to start.
            sleep_millis(MIN_TICK).await;
        }
    });
}

/// Sleeps for `duration` without pulling in a timer crate.
async fn sleep_millis(duration: Duration) {
    let millis = i32::try_from(duration.as_millis()).unwrap_or(i32::MAX);
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        let window = web_sys::window();
        if let Some(window) = window {
            let _ = window
                .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, millis.max(1));
        }
    });
    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
}
