//! A flag that says whether the component that created it is still mounted.
//!
//! Writing to a signal whose scope is gone panics, and on the browser that panic poisons
//! the single-threaded executor so every later task fails too. Async continuations that
//! touch component state guard their writes with [`Alive::if_alive`].

use std::cell::RefCell;

use leptos::prelude::on_cleanup;

thread_local! {
    static FLAGS: RefCell<Slots> = const { RefCell::new(Slots::new()) };
}

struct Slots {
    alive: Vec<bool>,
    free: Vec<usize>,
}

impl Slots {
    const fn new() -> Self {
        Self {
            alive: Vec::new(),
            free: Vec::new(),
        }
    }

    fn take(&mut self) -> usize {
        if let Some(slot) = self.free.pop() {
            self.alive[slot] = true;
            slot
        } else {
            self.alive.push(true);
            self.alive.len() - 1
        }
    }

    fn release(&mut self, slot: usize) {
        if let Some(alive) = self.alive.get_mut(slot)
            && *alive
        {
            *alive = false;
            self.free.push(slot);
        }
    }
}

/// Tracks whether the owning component is still mounted.
#[derive(Debug, Clone, Copy)]
pub struct Alive(usize);

impl Alive {
    /// Starts alive and flips to dead when the current reactive scope is cleaned up.
    ///
    /// Must be called inside a component body: outside any owner, `on_cleanup` has
    /// nothing to register with and the flag stays true forever.
    #[must_use]
    pub fn new() -> Self {
        let slot = FLAGS.with(|flags| flags.borrow_mut().take());
        on_cleanup(move || {
            FLAGS.with(|flags| flags.borrow_mut().release(slot));
        });
        Self(slot)
    }

    /// Whether the component is still mounted.
    #[must_use]
    pub fn is_alive(self) -> bool {
        FLAGS.with(|flags| flags.borrow().alive.get(self.0).copied().unwrap_or(false))
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
