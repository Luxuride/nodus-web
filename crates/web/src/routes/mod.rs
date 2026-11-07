//! Route views. One file per screen; every one of them renders SDK state through
//! [`crate::state::bridge`] and never talks to a transport or to storage.

pub mod account;
pub mod admin;
pub mod callback;
pub mod compose;
pub mod feed;
pub mod format;
pub mod home;
pub mod lists;
pub mod notifications;
pub mod people;
pub mod saved;
pub mod search;
pub mod settings;
pub mod sign_in;
pub mod status;
