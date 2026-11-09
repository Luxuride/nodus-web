//! Pieces that a page composes: no screen of their own, no I/O.
//!
//! The split this module makes is between *what a page is made of* (here) and *what a page is*
//! (`crate::pages`). A component may hold a `ChildrenFn` slot or a `Callback`, but it never
//! reads state, a clock, storage or the router — those arrive from the screen that owns them.

pub mod card_renderer;
pub mod chrome;
pub mod composer_card;
pub mod format;
pub mod frame;
pub mod icons;
pub mod lists;
pub mod media;
pub mod notification_card;
pub mod page_header;
pub mod panel;
pub mod props;
pub mod rail;
pub mod screen;
pub mod status;
pub mod timeline_tabs;

pub use card_renderer::CardRenderer;
