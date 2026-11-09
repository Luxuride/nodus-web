//! Presentation that more than one screen needs and has no single screen of its own.
//!
//! The drawings themselves are in `nodus-view`, beside the components that use them, so the
//! server's first paint and the mounted client cannot spell the same icon two ways.

pub mod drafts;
pub mod editor;
pub mod language;
