//! A counter that changes when the route changes.
//!
//! A component's cleanup runs both when the writer leaves the screen and when Leptos
//! rebuilds the view around a signal it reads. The draft keeper must only fire in the first
//! case: keeping on every rebuild stores each prefix of what is being typed.

use core::cell::Cell;

thread_local! {
    static EPOCH: Cell<u64> = const { Cell::new(0) };
}

/// The current navigation epoch.
pub fn get() -> u64 {
    EPOCH.with(Cell::get)
}

/// Marks a navigation, so cleanups that ran before it can tell the difference.
pub fn bump() {
    EPOCH.with(|epoch| epoch.set(epoch.get() + 1));
}
