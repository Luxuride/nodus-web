//! Media: the rules that decide a tile's size, and the grid that draws the tiles.
//!
//! The rules are separate from the grid because both paints need them and only one paints a
//! grid: the client sizes a single picture for the viewer, the server only ever draws the grid.

pub mod grid;
pub mod rules;

pub use rules::{attachments_label, box_style, extent, is_player, style_for, veil};
