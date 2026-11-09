//! The parts of the interface that both the browser and the server draw.
//!
//! This crate exists because of one bug that kept coming back: the server painted a page
//! that looked nothing like the one the client painted a moment later, so every load moved
//! the layout under the reader. Two templates cannot be kept in sync; one component tree
//! can. Everything here is pure markup — no `web_sys`, no signals that outlive a render, no
//! router — so it compiles for `wasm32` and for the server, and `leptos::ssr` can turn it
//! into the first paint while the client turns the same tree into the live one.

pub mod components;
pub mod pages;

// Components live under `components/`; these names are how both paints spell them, and how
// they were spelled before the move.
pub use components::chrome;
pub use components::chrome::tab_list::{Tab, TabList, TabListProps};
pub use components::composer_card::*;
pub use components::format::{describe_notification, display_name, relative_age, unix_seconds};
pub use components::frame::*;
pub use components::icons;
pub use components::lists;
pub use components::media::grid as media_grid;
pub use components::media::rules as media;
pub use components::notification_card::*;
pub use components::page_header::*;
pub use components::rail::*;
pub use components::screen::*;
pub use components::status::body as status_body;
pub use components::status::card as status_card;
pub use components::status::footer as status_footer;
pub use components::status::header as status_header;
pub use components::timeline_tabs::*;

/// The account page, at `crate::pages::accounts`.
pub use pages::accounts as profile;
/// The thread page, at `crate::pages::status`.
pub use pages::status as thread;
