//! The chrome around a screen, drawn once for both paints.
//!
//! These are the pieces `crates/server/src/render.rs` still builds as strings: the heading a
//! timeline opens with, the foot that asks for the next page, and the grey cards that hold
//! the space while a screen has nothing to show yet. Each is a single definition of the
//! markup, so the server's first paint and the client's mount put the same boxes in the same
//! places instead of moving the page under the reader.
//!
//! Nothing here reads the router, the clock or a transport: what a component would otherwise
//! look up arrives as a prop, and an interaction that does not exist on one of the paints
//! arrives as an absent callback rather than as different markup.

pub mod more_button;
pub mod skeleton;
pub mod tab_list;
pub mod timeline_header;

pub use more_button::MoreButton;
pub use skeleton::Skeleton;
pub use tab_list::{Tab, TabList};
pub use timeline_header::TimelineHeader;
