//! The four rows the server's paint and the client's mount share.
//!
//! Each of these was a string built in `crates/server/src/render.rs` and a hand-written view
//! in `crates/web/src/routes`, and the two drifted: a badge said "unread" on one side and
//! "new" on the other, a control existed on one side and not the other, and a row the server
//! could not enrich changed shape instead of losing its contents. A row here is the markup
//! both paints agree on. What still differs between them is data the caller passes in (a
//! bio, a formatted time) and interactions it may or may not own.
//!
//! The rules every row follows:
//!
//! - Time arrives as a `String` prop. This crate reads no clock, no URL and no storage.
//! - An interaction is an `Option<Callback<..>>`. Without one the row keeps the control and
//!   disables it, so the two paints do not differ by a control's width.
//! - A fragment only the client has is an `Option<ChildrenFn>` slot, and it renders nothing
//!   at all when it is absent.
//! - Missing data is a state, not a different row: the same boxes, with the unavailable
//!   branch inside them.

pub mod account_row;
pub mod rows;
pub mod conversation_row;
pub mod list_row;
pub mod scheduled_row;

pub use account_row::AccountRow;
pub use rows::Rows;
pub use conversation_row::ConversationRow;
pub use list_row::ListRow;
pub use scheduled_row::ScheduledRow;
