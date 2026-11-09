//! The page bodies both paints draw, from data and callbacks they are handed.
//!
//! `nodus-ui` holds the pieces; this crate composes them into screens. It knows nothing
//! about routes, the runtime or a transport.

pub mod pages;
pub mod props;

pub use nodus_ui::*;

/// The account page, at `crate::pages::accounts`.
pub use pages::accounts as profile;
/// The thread page, at `crate::pages::status`.
pub use pages::status as thread;
