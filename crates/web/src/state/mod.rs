//! Client state: the SDK, and the bridge that makes it observable by Leptos.
//!
//! Views go through this module rather than talking to `nodus-sdk`
//! directly, so the single-threaded bridge stays the only place where the
//! framework and the state machine meet.

pub mod bridge;
