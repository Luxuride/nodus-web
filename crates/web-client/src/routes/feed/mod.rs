//! Feed cards and their text helpers.

mod actions;
mod item;
pub mod posts;
mod reason;
pub(crate) mod share;

pub(crate) use item::FeedItem;
pub(crate) use posts::card::PostCard;
pub(crate) use reason::degraded_line;
