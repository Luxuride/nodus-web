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
use std::time::{Duration, Instant};

use leptos::task::spawn_local;
use nodus_sdk::Driver;

use super::SDK;

/// Longest the loop sleeps when the driver has no opinion.
///
/// Without a deadline the only useful thing to do is check again later: a tab can be
/// hidden for a long time, and a minute costs nothing because nothing runs.
const IDLE_SLEEP: Duration = Duration::from_secs(60);

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
        loop {
            let due = DRIVER.with_borrow(|slot| {
                slot.as_ref()
                    .and_then(|driver| driver.next_wake(Instant::now()))
            });

            let sleep_for = due.map_or(IDLE_SLEEP, |wait| wait.min(IDLE_SLEEP));
            if !sleep_for.is_zero() {
                sleep_millis(sleep_for).await;
                continue;
            }

            // Due. Take the driver out so the SDK can borrow it mutably while the
            // state handles are borrowed elsewhere; put it back before awaiting
            // anything else.
            let Some(mut driver) = DRIVER.with_borrow_mut(Option::take) else {
                return;
            };
            let Some(sdk) = SDK.with_borrow(std::clone::Clone::clone) else {
                DRIVER.with_borrow_mut(|slot| *slot = Some(driver));
                return;
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
            sleep_millis(sleep_for).await;
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
