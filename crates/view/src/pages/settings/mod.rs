//! The settings screens: one card per question, the same marks on both paints.
//!
//! A card owns its boxes and its sentences; the caller owns every signal, every write and
//! every clock. That is why the pieces here take `Signal`s and `Callback`s rather than
//! reading a store: the client keeps the state it already has, and the server can paint the
//! same tree from `*PageProps`.
//!
//! Only the blocks that are still markup live here so far: the instance's identity and
//! rules, the background-sync rows and its consent dialog, the account, clock, feed and
//! list cards, and the tab strip and page shell that hold them.

pub mod account;
pub mod feeds;
pub mod form;
pub mod instance;
pub mod instance_card;
pub mod lists;
pub mod nav;
pub mod page;
pub mod profile;
pub mod sync;
