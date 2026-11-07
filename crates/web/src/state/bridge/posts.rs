//! The post half of the Feed tab: loading pages of recommended posts.
//!
//! Mirrors `bridge::feed`, with one difference that comes from the server: a post
//! page is never served from a reused snapshot, so "refresh" here really does ask
//! for new work, and the cursor is the only way to continue a list.

use leptos::prelude::*;

use super::{handle, impressions, with_sdk};

/// UI state for the post list, kept apart from [`FeedUi`](super::FeedUi) so the two
/// tabs cannot dim or error each other.
#[derive(Debug, Clone, Copy)]
pub struct PostsUi {
    /// A request is in flight; the list stays visible, dimmed.
    pub refreshing: RwSignal<bool>,
    /// The last request's error, rendered inline with a retry.
    pub error: RwSignal<Option<String>>,
}

impl PostsUi {
    fn new() -> Self {
        Self {
            refreshing: RwSignal::new(false),
            error: RwSignal::new(None),
        }
    }
}

thread_local! {
    static POSTS_UI: PostsUi = PostsUi::new();
}

/// Creates the post list's UI state before any component owns it.
///
/// The signals are a `thread_local`, so they are created on first use — and the first use
/// is the Posts component. That made the component their owner: navigating away disposed
/// them, and the second visit read disposed handles
/// (`panicked at .../reactive_graph-0.2.16/src/traits.rs:394: Tried to access a reactive
/// value that has already been disposed`). Touching the slot from `install_reactive_root`
/// gives them no owner at all, so they live as long as the page does.
pub fn install() {
    POSTS_UI.with(|_| ());
}

/// The post list's UI state.
#[must_use]
pub fn posts_ui() -> PostsUi {
    POSTS_UI.with(|ui| *ui)
}

/// Loads one page of recommended posts, keeping the current list visible.
///
/// `cursor` is `None` for a refresh and the previous page's `next_cursor` for
/// "Load more"; the SDK decides whether the page replaces or appends, because only
/// it knows whether the caller passed a cursor.
pub fn reload_posts(cursor: Option<String>) {
    let Some(sdk) = handle() else {
        return;
    };
    let ui = posts_ui();
    ui.refreshing.set(true);
    ui.error.set(None);
    wasm_bindgen_futures::spawn_local(async move {
        match sdk.load_posts(cursor.as_deref()).await {
            Ok(feed) => {
                ui.error.set(None);
                impressions::set_feed(&feed.feed_id);
            }
            Err(error) => ui.error.set(Some(error.to_string())),
        }
        ui.refreshing.set(false);
    });
}

/// Loads the first post page for a signed-in session.
///
/// Called from the tab's `on_mount`: a list that loads only when the user clicks
/// "Refresh" looks broken the first time it is opened.
pub fn load_posts_once() {
    let already = with_sdk(|sdk| !sdk.state().posts.get().items.is_empty());
    if !already {
        reload_posts(None);
    }
}
