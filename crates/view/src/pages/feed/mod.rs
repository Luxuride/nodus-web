//! The Feed tab: what the recommender suggests, and nothing else.
//!
//! The tab itself (its two lists and the switch between them) is client state, so the
//! page here is the material it is made of: one recommended post, drawn the same way
//! whether the browser or the server paints it.

pub mod posts;

pub mod accounts;
