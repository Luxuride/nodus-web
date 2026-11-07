//! Route views. One file per screen; every one of them renders SDK state through
//! [`crate::state::bridge`] and never talks to a transport or to storage.

pub mod callback;
pub mod feed;
pub mod settings;
pub mod sign_in;
