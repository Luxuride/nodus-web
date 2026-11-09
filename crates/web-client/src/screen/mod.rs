//! One load, one place for its three answers.
//!
//! Every screen in `routes/` did the same four things by hand: a `signal` for the value, one
//! for the error, one for "a request is in flight", and an `Alive` guard so a late answer
//! cannot write into a screen that is gone. Repeating that is what made a 40-line screen out
//! of a 40-line fetch — and forgetting the guard is what poisoned the js-sys executor.
//!
//! [`spawn_into`] holds the two writes that every one of those screens needs and that are
//! easy to get wrong: clear the error, and always write "the request finished" — inside the
//! `Alive` guard, on both outcomes. A screen keeps its *own* state (which tab, which cursor,
//! which filters); only the transport's answer is handled here.

use std::future::Future;

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::state::alive::Alive;

/// Runs one request, reports its failure into a screen's error signal, and hands a success to
/// `on_ok` — inside the same `Alive` guard.
///
/// The guard is the point: `on_ok` runs after the `.await`, so anything it writes is a write
/// to a signal that may already belong to a screen the reader has left. A missing guard there
/// is the panic that poisons the js-sys executor.
pub fn spawn_with<T, F, D>(
    error: WriteSignal<Option<String>>,
    done: WriteSignal<bool>,
    request: F,
    on_ok: D,
) where
    T: 'static,
    F: Future<Output = Result<T, String>> + 'static,
    D: FnOnce(T) + 'static,
{
    let alive = Alive::new();
    error.set(None);
    spawn_local(async move {
        let outcome = request.await;
        alive.if_alive(|| {
            match outcome {
                Ok(value) => on_ok(value),
                Err(message) => error.set(Some(message)),
            }
            done.set(false);
        });
    });
}

/// [`spawn_with`] for a request whose success carries nothing the screen needs.
pub fn spawn_into<F>(error: WriteSignal<Option<String>>, done: WriteSignal<bool>, request: F)
where
    F: Future<Output = Result<(), String>> + 'static,
{
    spawn_with(error, done, request, |()| ());
}
