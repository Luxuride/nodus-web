//! Recommended posts: the second half of the Feed tab.
//!
//! Renders `PostFeedState` and nothing else. The text arrives as the instance's
//! HTML; until the sanitizer from `web/docs/04-ui.md` exists in this crate, it is
//! reduced to plain text in [`text`] rather than injected with `inner_html` — a
//! `dangerously_set_inner_html` call is exactly the XSS surface the token-in-the-
//! browser design cannot afford.

use leptos::prelude::*;

use crate::state::bridge::{load_posts_once, posts_ui, reload_posts, revision, with_sdk};

mod card;
pub mod text;

pub use self::text::plain_text;

use self::card::PostCard;

/// The list of recommended posts.
#[component]
pub fn Posts() -> impl IntoView {
    load_posts_once();

    let revision = revision();
    let ui = posts_ui();

    let items = move || {
        revision.get();
        with_sdk(|sdk| sdk.state().posts.get().items)
    };
    let next_cursor = move || {
        revision.get();
        with_sdk(|sdk| sdk.state().posts.get().next_cursor)
    };

    view! {
        <div class="mb-4 flex items-baseline justify-between gap-4">
            <h2 class="text-sm font-medium opacity-70">"Posts to read"</h2>
            <button
                class="btn btn-ghost btn-sm gap-2"
                disabled=move || ui.refreshing.get()
                on:click=move |_| reload_posts(None)
            >
                <Show when=move || ui.refreshing.get()>
                    <span class="loading loading-spinner loading-xs"></span>
                </Show>
                "Refresh"
            </button>
        </div>

        {move || ui.error.get().map(|message| view! {
            <div class="alert alert-soft alert-error mb-4" role="alert">
                <span>{message}</span>
                <button class="btn btn-sm" on:click=move |_| reload_posts(None)>
                    "Retry"
                </button>
            </div>
        })}

        {move || {
            (items().is_empty() && !ui.refreshing.get() && ui.error.get().is_none())
                .then(|| view! { <EmptyPosts/> })
        }}

        <ul class="space-y-3 transition-opacity" class:opacity-50=move || ui.refreshing.get()>
            <For each=items key=|item| item.item_id let:item>
                <PostCard item/>
            </For>
        </ul>

        {move || next_cursor().map(|cursor| view! {
            <div class="mt-6">
                <button
                    class="btn btn-outline btn-sm w-full"
                    disabled=move || ui.refreshing.get()
                    on:click=move |_| reload_posts(Some(cursor.clone()))
                >
                    {move || if ui.refreshing.get() { "Loading…" } else { "Load more posts" }}
                </button>
            </div>
        })}
    }
}

/// Why there is nothing to read, and what changes it.
#[component]
fn EmptyPosts() -> impl IntoView {
    view! {
        <div class="card border border-dashed border-base-300 bg-base-100">
            <div class="card-body items-center gap-2 py-10 text-center">
                <h2 class="card-title text-base">"No posts yet"</h2>
                <p class="max-w-sm text-sm opacity-70">
                    "Posts come from the accounts the recommender already knows something about, so this list fills in after the first background sync has fetched some timelines."
                </p>
            </div>
        </div>
    }
}
