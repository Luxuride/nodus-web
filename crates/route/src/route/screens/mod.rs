#![expect(
    clippy::needless_pass_by_value,
    reason = "Leptos components receive their props by value"
)]

//! Route bodies shared by both paints.

pub mod account;
pub mod admin;
pub mod compose;
pub mod directory;
pub mod feed;
pub mod lists;
pub mod messages;
pub mod more;
pub mod notifications;
pub mod people;
pub mod public_account;
pub mod saved;
pub mod scheduled;
pub mod search;
pub mod settings;
pub mod thread;
