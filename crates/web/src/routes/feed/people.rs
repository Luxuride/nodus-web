//! Accounts to follow: the original Feed tab body.
//!
//! This view renders `FeedState` and nothing else: no ranking, no HTTP, no JSON.
//! Each action is either a state change (dismiss) or a single instance write
//! (follow, unfollow), both funnelled through [`crate::state::bridge`].
//!
//! Four states are rendered explicitly: empty, error, refreshing (the list stays
//! visible, dimmed) and degraded (one muted line, because a weaker ranking is not
//! a failure).

use leptos::prelude::*;

use crate::router::{Route, link};
use crate::state::bridge::{feed_ui, reload_feed, revision, with_sdk};

use crate::routes::feed::item::FeedItem;
use crate::routes::feed::reason::degraded_line;

/// The list of recommended accounts.
#[component]
pub fn People() -> impl IntoView {
    let revision = revision();
    let ui = feed_ui();

    let items = move || {
        revision.get();
        with_sdk(|sdk| {
            sdk.state()
                .feed
                .with(|feed| feed.visible().cloned().collect::<Vec<_>>())
        })
    };
    let degraded = move || {
        revision.get();
        with_sdk(|sdk| sdk.state().feed.get().degraded)
    };
    let next_cursor = move || {
        revision.get();
        with_sdk(|sdk| sdk.state().feed.get().next_cursor)
    };

    view! {
        <div class="mb-4 flex items-baseline justify-between gap-4">
            <h2 class="text-sm font-medium opacity-70">"People to follow"</h2>
            <button
                class="btn btn-ghost btn-sm gap-2"
                disabled=move || ui.refreshing.get()
                on:click=move |_| reload_feed(None)
            >
                <Show when=move || ui.refreshing.get()>
                    <span class="loading loading-spinner loading-xs"></span>
                </Show>
                "Refresh"
            </button>
        </div>

        {move || degraded().first().copied().map(|flag| view! {
            <div class="alert alert-soft alert-info mb-4 py-2 text-sm" role="status">
                <span>{degraded_line(flag)}</span>
            </div>
        })}

        {move || ui.error.get().map(|message| view! {
            <div class="alert alert-soft alert-error mb-4" role="alert">
                <span>{message}</span>
                <button class="btn btn-sm" on:click=move |_| reload_feed(None)>
                    "Retry"
                </button>
            </div>
        })}

        {move || {
            (items().is_empty() && !ui.refreshing.get() && ui.error.get().is_none())
                .then(|| view! { <EmptyFeed/> })
        }}

        <ul class="space-y-3 transition-opacity" class:opacity-50=move || ui.refreshing.get()>
            // The follow flag is part of the key: undoing a follow flips it in
            // place, and remounting is what makes the card show "Follow" again
            // without a full page reload.
            <For each=items key=|item| (item.item_id, item.account.already_following) let:item>
                <FeedItem item/>
            </For>
        </ul>

        {move || next_cursor().map(|cursor| view! {
            <div class="mt-6">
                <button
                    class="btn btn-outline btn-sm w-full"
                    disabled=move || ui.refreshing.get()
                    on:click=move |_| reload_feed(Some(cursor.clone()))
                >
                    {move || if ui.refreshing.get() { "Loading…" } else { "Load more people" }}
                </button>
            </div>
        })}
    }
}

/// Why the list is empty, and the one thing that changes it.
#[component]
fn EmptyFeed() -> impl IntoView {
    view! {
        <div class="card border border-dashed border-base-300 bg-base-100">
            <div class="card-body items-center gap-2 py-10 text-center">
                <h2 class="card-title text-base">"No recommendations yet"</h2>
                <p class="max-w-sm text-sm opacity-70">
                    "Follow a few accounts to give the recommender something to work with, or turn on background sync in "
                    <a
                        class="link link-primary"
                        href=Route::Settings(crate::router::SECTION_DEFAULT.to_owned()).path()
                        on:click=move |event| link(event, Route::Settings(crate::router::SECTION_DEFAULT.to_owned()))
                    >
                        "Settings"
                    </a>
                    " so it can learn from your activity."
                </p>
            </div>
        </div>
    }
}
