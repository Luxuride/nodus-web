//! The icons, defined once in `nodus-view`.
//!
//! The server draws the rail before any wasm runs, so the drawings have to live in the
//! crate both paints share; re-exporting keeps every `use crate::ui::icons::Icon` working.

pub use nodus_view::icons::*;
