//! The vocabulary both artifacts share.
//!
//! The client is the package `nodus-web-client` (it owns the DOM code and the wasm), the
//! server is `nodus-web-server` (it owns the HTTP host). Each is binary-only, so a workspace
//! build can never enable `leptos/csr` and `leptos/ssr` at the same time.
//!
//! What is left here is the part both must agree on: the route table and the addresses it
//! maps to. It names neither `web_sys` nor axum, so it compiles for both targets.

pub mod shared;

pub use shared::router::{Route, ADMIN_DEFAULT, BASE, CALLBACK_PATH, MODERATION_DEFAULT};
