//! One module per screen: the column a reader lands on, from its heading to its error line.
//!
//! A page owns what a screen *is* — the column, the heading, the list container, the empty
//! state, the paging control — and asks its caller for the two things it may not have: the
//! data, and the verbs. It reads no address, no clock, no storage and no instance, so the same
//! page paints from the server and mounts on the client.
//!
//! The tree mirrors the addresses the screens answer on, so a reader who knows the URL knows
//! the module: `/notifications` is `notifications`, and a screen with parts keeps them in the
//! same directory as submodules rather than in a shared cupboard.

// The tree mirrors the addresses, including the parts that vary: `[id]` and `[shelf]` are
// the dynamic segments, declared with `#[path]` because a segment is not an identifier.
#[path = "accounts/[id]/mod.rs"]
pub mod account;
pub use account as accounts;

pub mod directory;
pub mod lists;
pub mod messages;
#[path = "moderation/[section]/mod.rs"]
pub mod moderation;
pub mod notifications;
pub use moderation as people;
#[path = "saved/[shelf]/mod.rs"]
pub mod saved_shelf;
pub use saved_shelf as saved;

pub mod scheduled;
pub mod search;
#[path = "status/[id]/mod.rs"]
pub mod status_page;
pub use status_page as status;
