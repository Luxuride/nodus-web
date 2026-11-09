//! Code both paints compile: the route vocabulary the client navigates by and the server
//! resolves URLs against.
//!
//! Nothing here may name `web_sys`, `gloo`, `wasm_bindgen` or axum — that is the whole
//! admission rule, and it is what makes this folder build under `hydrate` and `ssr` alike.

pub mod router;
pub mod screens;
