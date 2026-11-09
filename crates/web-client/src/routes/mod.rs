//! Route views. One file per screen; every one of them renders SDK state through
//! [`crate::state::bridge`] and never talks to a transport or to storage.

#[cfg(feature = "admin")]
pub mod admin;
pub mod callback;
pub mod compose;
pub mod feed;
pub mod filtering;
pub mod format;
pub mod home;
#[cfg(feature = "admin")]
pub mod moderation;
pub mod settings;
pub mod sign_in;
pub mod status;
