//! The wasm client: everything that paints in a browser, compiled only for `hydrate`.
//!
//! Every module here names `web_sys`, `js_sys`, `gloo`, `BrowserInstance` or a DOM signal,
//! which is why the server build does not include this folder. `app` is the shell and its
//! screens, `boot` installs them, and `state` holds the browser-side bridge into the SDK.
//! `lib.rs` re-exports the modules at the crate root, so the screens keep naming their
//! siblings as `crate::state::…` rather than through this folder.

pub mod app;
pub mod boot;
pub mod media;
pub mod platform;
pub mod public;
pub mod router;
pub mod routes;
pub mod scroll;
pub mod shell;
pub mod state;
pub mod theme;
pub mod ui;
