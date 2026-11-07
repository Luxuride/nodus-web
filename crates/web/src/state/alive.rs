//! A flag that says whether the component that created it is still mounted.
//!
//! Writing to a signal whose scope is gone is a **panic**, not a no-op:
//! `reactive_graph` reports `Tried to access a reactive value that has already been
//! disposed`, and because the write happens inside a task poll, the unwind leaves
//! `js-sys`'s single-threaded executor with its `RefCell` borrowed — every later task
//! then panics with `RefCell already borrowed` and the app stops reacting for good.
//!
//! So every async continuation guards its writes with [`Alive::if_alive`].
//!
//! # Why a handle and not an `Rc<Cell<bool>>`
//!
//! The first attempt held `Rc<Cell<bool>>`, which failed twice over: `view!` and
//! `Callback::new` require `Send + Sync` closures (signals are both), and a non-`Copy`
//! guard cannot be handed to two handlers in one `view!` — every component that tried
//! got `expected a closure that implements the Fn trait` or `cannot be sent between
//! threads safely`. The handle is an index into a thread-local table instead: `Copy`,
//! `Send`, `Sync`, and the entry is cleared by `on_cleanup`, which is the only thing
//! that knows the screen is gone.

use std::cell::RefCell;

use leptos::prelude::on_cleanup;

thread_local! {
    /// One slot per live component; `false` once its scope was cleaned up.
    static FLAGS: RefCell<Vec<bool>> = const { RefCell::new(Vec::new()) };
}

/// Tracks whether the owning component is still mounted.
#[derive(Debug, Clone, Copy)]
pub struct Alive(usize);

impl Alive {
    /// Starts alive and flips to dead when the current reactive scope is cleaned up.
    ///
    /// Must be called inside a component body (or inside an `Effect`, which has its own
    /// owner): outside any owner, `on_cleanup` has nothing to register with and the flag
    /// would stay true forever.
    pub fn new() -> Self {
        let slot = FLAGS.with(|flags| {
            let mut flags = flags.borrow_mut();
            flags.push(true);
            flags.len() - 1
        });
        on_cleanup(move || {
            FLAGS.with(|flags| {
                if let Some(alive) = flags.borrow_mut().get_mut(slot) {
                    *alive = false;
                }
            });
        });
        Self(slot)
    }

    /// Whether the component is still mounted.
    pub fn is_alive(self) -> bool {
        FLAGS.with(|flags| flags.borrow().get(self.0).copied().unwrap_or(false))
    }

    /// Runs `write` only while the component is mounted.
    pub fn if_alive(self, write: impl FnOnce()) {
        if self.is_alive() {
            write();
        }
    }
}

impl Default for Alive {
    fn default() -> Self {
        Self::new()
    }
}
