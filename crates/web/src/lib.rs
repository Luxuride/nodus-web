//! The Nodus web crate: one tree, three folders, two artifacts.
//!
//! `browser` is the wasm client `trunk` packages (`hydrate`, the default); `server` is the
//! `nodus-web-server` binary that paints the first page (`ssr`); `shared` is the code both
//! compile. The folders are the gate: a module that names `web_sys` or `gloo` lives in
//! `browser`, one that names axum lives in `server`, and only code free of both belongs in
//! `shared`.
//!
//! The two artifacts are separate builds, never one build with both: the client is
//! `trunk build` (or `cargo build --features hydrate`), the server is
//! `cargo build -p nodus-web --no-default-features --features ssr --bin nodus-web-server`.
//! `admin` composes with either — `trunk build --features admin`, and
//! `--no-default-features --features "ssr,admin"` for a server that carries the screens.
//!
//! This crate renders; it does not decide. All state lives in `nodus-sdk` (the sibling
//! `sdk` project), and every change is a `Command` there. Scope, data layer and
//! performance budget are specified in `docs/` in this repository; the SDK is specified in
//! `sdk/README.md`.

#[cfg(feature = "hydrate")]
pub mod browser;

pub mod shared;

#[cfg(feature = "ssr")]
pub mod server;

#[cfg(feature = "hydrate")]
pub use browser::boot::start;

// The pre-split names stay valid: the screens name their siblings through the crate root,
// and moving them under `browser` was a folder change, not a rewrite of every import.
#[cfg(feature = "hydrate")]
pub use browser::{
    app, boot, media, platform, public, router, routes, scroll, shell, state, theme, ui,
};
