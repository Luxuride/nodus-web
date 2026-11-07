//! Route views. One file per screen; every one of them renders SDK state through
//! [`crate::state::bridge`] and never talks to a transport or to storage.

pub mod account;
pub mod callback;
pub mod feed;
pub mod format;
pub mod home;
pub mod notifications;
pub mod search;
pub mod settings;
pub mod sign_in;
pub mod status;
