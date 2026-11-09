//! The settings screens: one card per question, the same marks on both paints.
//!
//! A card owns its boxes and its sentences; the caller owns every signal, every write and
//! every clock. That is why the pieces here take `Signal`s and `Callback`s rather than
//! reading a store: the client keeps the state it already has, and the server can paint the
//! same tree from `*PageProps`.
//!
//! Only the blocks that are still markup live here so far: the instance's identity and
//! rules, the background-sync rows and its consent dialog, and the form rows the settings
//! cards are built from. The cards themselves still wrap `components::panel::Panel` on the
//! client, which is the arrangement the caller is free to change.

pub mod form;
pub mod instance;
pub mod sync;
