//! The recommendation Feed tab.
//!
//! This view renders `FeedState` and nothing else: no ranking, no HTTP, no JSON.
//! Each action is either a state change (dismiss) or a single instance write
//! (follow), both funnelled through [`crate::state::bridge`].
//!
//! Four states are rendered explicitly: empty, error, refreshing (the list stays
//! visible, dimmed) and degraded (one muted line, because a weaker ranking is not
//! a failure).

use leptos::prelude::*;
use nodus_sdk::Degraded;
use nodus_shared::{AccountItem, Reason};

use crate::router::{link, Route};
use crate::state::bridge::{feed_ui, follow_item, not_interested, reload_feed, revision, with_sdk};

/// The Feed tab.
#[component]
pub fn Feed() -> impl IntoView {
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
        <section class="mx-auto w-full max-w-2xl px-4 py-6">
            <div class="mb-5 flex items-baseline justify-between gap-4">
                <h1 class="text-xl font-semibold tracking-tight">"People to follow"</h1>
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

            <ul
                class="space-y-3 transition-opacity"
                class:opacity-50=move || ui.refreshing.get()
            >
                <For each=items key=|item| item.item_id let:item>
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
                        {move || if ui.refreshing.get() { "Loading…" } else { "Load more" }}
                    </button>
                </div>
            })}
        </section>
    }
}

/// One ranked account: who it is, why it is here, and the two actions.
#[component]
fn FeedItem(item: AccountItem) -> impl IntoView {
    let item_id = item.item_id;
    let account_id = item.account.id.clone();
    let acct = item.account.acct.clone();
    let display_name = if item.account.display_name.trim().is_empty() {
        acct.clone()
    } else {
        item.account.display_name.clone()
    };
    let note = item.account.note.clone();
    let reason = reason_line(&item.reason);
    let followers = follower_label(item.account.followers_count);
    let already_following = item.account.already_following;

    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);

    let on_follow = {
        let account_id = account_id.clone();
        move |_| {
            if busy.get_untracked() {
                return;
            }
            busy.set(true);
            error.set(None);
            let account_id = account_id.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let result = follow_item(item_id, &account_id).await;
                busy.set(false);
                if let Err(message) = result {
                    error.set(Some(message));
                }
            });
        }
    };
    let on_not_interested = move |_| {
        if busy.get_untracked() {
            return;
        }
        busy.set(true);
        error.set(None);
        wasm_bindgen_futures::spawn_local(async move {
            let result = not_interested(item_id).await;
            busy.set(false);
            if let Err(message) = result {
                error.set(Some(message));
            }
        });
    };

    view! {
        <li class="card border border-base-300 bg-base-100 shadow-sm">
            <div class="card-body gap-3 p-4">
                <div class="flex items-start justify-between gap-3">
                    <div class="flex min-w-0 items-center gap-3">
                        <div class="avatar">
                            <div class="w-10 rounded-full bg-base-300">
                                <img src=item.account.avatar_url alt="" referrerpolicy="no-referrer" />
                            </div>
                        </div>
                        <div class="min-w-0">
                            <p class="truncate font-semibold leading-tight">{display_name}</p>
                            <p class="truncate text-sm opacity-60">{format!("@{acct}")}</p>
                        </div>
                    </div>
                    <span class="badge badge-ghost badge-sm whitespace-nowrap">{followers}</span>
                </div>
                // The reason is the product: an item the user cannot explain is an
                // item they will not trust. It is a badge, not body text, so it reads
                // as metadata rather than as the account's own description.
                <div>
                    <span class="badge badge-soft badge-primary badge-sm">{reason}</span>
                </div>
                {(!note.trim().is_empty()).then(|| view! {
                    <p class="line-clamp-3 text-sm opacity-80">{note}</p>
                })}
                <div class="card-actions justify-end gap-2">
                    <button
                        class="btn btn-ghost btn-sm"
                        disabled=move || busy.get()
                        on:click=on_not_interested
                    >
                        "Not interested"
                    </button>
                    <button
                        class="btn btn-primary btn-sm"
                        disabled=move || busy.get() || already_following
                        on:click=on_follow
                    >
                        <Show when=move || busy.get()>
                            <span class="loading loading-spinner loading-xs"></span>
                        </Show>
                        {if already_following { "Following" } else { "Follow" }}
                    </button>
                </div>
                {move || error.get().map(|message| view! {
                    <p class="text-sm text-error" role="alert">{message}</p>
                })}
            </div>
        </li>
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
                        href=Route::Settings.path()
                        on:click=move |event| link(event, Route::Settings)
                    >
                        "Settings"
                    </a>
                    " so it can learn from your activity."
                </p>
            </div>
        </div>
    }
}

/// One line of text for the first active degradation flag.
///
/// Muted body text, not a banner: a weaker ranking is a caveat about the list, not
/// an error that stops the user.
fn degraded_line(reason: Degraded) -> &'static str {
    match reason {
        Degraded::ColdStart => "Too few signals yet: these are the instance's cold-start picks.",
        Degraded::StaleSimilarity => "The similarity data is a few hours old, so the order may be off.",
        Degraded::NoServerSync => "Background sync is off, so these are ranked from this session only.",
    }
}

/// Turns the server's machine code into one line of text.
///
/// The codes come from the recommender and can grow without a client release, so
/// an unknown code must never reach the user: the fallback says what every item in
/// the list has in common instead of printing `followed_by_similar_users`.
fn reason_line(reason: &Reason) -> String {
    match reason.code.as_str() {
        "followed_by_similar_users" | "similar_user" => similar_line(reason.support),
        "multiple_signals" => "Matched by more than one signal".to_owned(),
        "affinity" => "Similar to accounts you engage with".to_owned(),
        "popular_fallback" | "popular" => "Popular on your instance".to_owned(),
        "author_you_should_follow" => "Author of posts you engage with".to_owned(),
        _ => "Recommended for you".to_owned(),
    }
}

/// "Followed by N people with similar taste", with the count left out when it adds
/// nothing.
fn similar_line(support: u8) -> String {
    match support {
        0 | 1 => "Followed by someone with similar taste".to_owned(),
        count => format!("Followed by {count} people with similar taste"),
    }
}

/// The follower count, singular or plural.
fn follower_label(count: i64) -> String {
    match count {
        1 => "1 follower".to_owned(),
        _ => format!("{count} followers"),
    }
}
