//! The mount flag lives in the shared crate because async continuations on both paints
//! need the same guarantee.

pub use nodus_view::Alive;
