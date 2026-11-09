//! The status card, split into its parts.
//!
//! A card is a header, a body and a footer, and each part is asked for separately because the
//! paints differ in the middle: the server inlines sanitised HTML, the client mounts a live
//! body with a media viewer and a click that opens the thread. The header and the footer are
//! the same in both, so they are the parts worth sharing.

pub mod body;
pub mod card;
pub mod footer;
pub mod header;
pub mod menu;
pub mod share_button;
