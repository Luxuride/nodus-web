//! Entry point of the wasm client.
//!
//! What it does lives in `nodus_web`; this file is the binary `trunk` builds. The module
//! tree, the installs and the mount are in `src/lib.rs`, `src/browser/boot.rs` and
//! `src/browser/app.rs` because the server binary in `src/bin/` shares the crate, not this
//! file.

/// Boots the client.
#[cfg(feature = "hydrate")]
fn main() {
    nodus_web::start();
}
