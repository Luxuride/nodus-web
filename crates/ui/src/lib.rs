#![recursion_limit = "512"]

//! The pieces a page composes: pure markup, no I/O.
//!
//! Everything here compiles for `wasm32` and for the server. Pages live in
//! `nodus-pages`; routing and the runtime live in `nodus-route`.

pub mod blurhash;
pub mod components;
pub mod mfm;

pub use components::anchor::{Anchor, Relative};
pub use components::chrome;
pub use components::chrome::tab_list::{Tab, TabList, TabListProps};
pub use components::chrome::{AppFooter, Note};
pub use components::composer_card::*;
pub use components::field_box::FieldBox;
pub use components::format::{describe_notification, display_name, relative_age, unix_seconds};
pub use components::frame::*;
pub use components::icons;
pub use components::language_picker::LanguagePicker;
pub use components::lists;
pub use components::media::grid as media_grid;
pub use components::media::lightbox::Lightbox;
pub use components::media::rules as media;
pub use components::notification_card::*;
pub use components::page_header::*;
pub use components::popover::Popover;
pub use components::rail::*;
pub use components::screen::*;
pub use components::shell::*;
pub use components::status::body as status_body;
pub use components::status::card as status_card;
pub use components::status::footer as status_footer;
pub use components::status::header as status_header;
pub use components::status::menu as status_menu;
pub use components::status::share_button as status_share;
pub use components::timeline_tabs::*;
