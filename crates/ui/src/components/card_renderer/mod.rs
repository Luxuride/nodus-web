//! How a page asks its caller to paint one status.

use leptos::prelude::AnyView;
use nodus_instance::model::Status;

/// Paints one status inside a list the page owns.
///
/// The seam exists because the two paints disagree about what a post *is*, and only about that:
/// the client has verbs, filters, a media viewer and a clock; the server has none of those and
/// wants the same card with them inert. The page keeps the `<ul>`, the keys, the empty state and
/// Load more; this decides the body of each `<li>`.
///
/// `Fn`, not `FnOnce`: a list repaints. Take the status by value — a card that borrowed would
/// make every caller's signal read outlive the row.
pub type CardRenderer = std::sync::Arc<dyn Fn(Status) -> AnyView + Send + Sync>;
