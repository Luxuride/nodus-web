//! One recommended post.
//!
//! The body and the age arrive already prepared: the body because the server has the
//! instance's HTML and the client has to strip it, and the age because a clock is not
//! state this crate may read (the same reason `PageHeader` takes its counters).

use leptos::prelude::*;
use nodus_instance::model::{Account, Status, Visibility};
use nodus_shared::PostItem;

use crate::status_card::StatusCard;

/// A post card: who wrote it, what it says, and where to read the original.
///
/// `on_seen` and `on_left` are how the caller records an impression (written when the
/// card leaves the screen, so it can carry dwell). The server passes neither and the
/// card renders identically without them.
#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "a Leptos component receives its props by value"
)]
pub fn PostCard(
    item: PostItem,
    body: String,
    age: String,
    #[prop(optional)] on_seen: Option<Callback<i64>>,
    #[prop(optional)] on_left: Option<Callback<i64>>,
) -> impl IntoView {
    if let Some(seen) = on_seen {
        seen.run(item.item_id);
    }
    if let Some(left) = on_left {
        on_cleanup(move || left.run(item.item_id));
    }

    let status = status_of(&item, body);
    let author_path = format!("https://{}/@{}", item.instance_host, item.account_acct);
    let thread_path = format!(
        "https://{}/@{}/{}",
        item.instance_host, item.account_acct, item.status_id
    );

    view! {
        <StatusCard
            status=status
            age=age
            author_path=author_path
            thread_path=thread_path
        />
    }
}

/// Draws a recommendation in the same shape as a timeline status.
///
/// The recommendation carries its author, body, time and counts but not the status's media,
/// poll or emoji, so those fields stay empty; the card is the timeline's card, not a second
/// layout that would drift from it.
fn status_of(item: &PostItem, body: String) -> Status {
    Status {
        id: item.status_id.clone(),
        created_at: item.published_at.clone(),
        content: body,
        visibility: Visibility::Public,
        account: Account {
            id: item.account_acct.clone(),
            acct: item.account_acct.clone(),
            display_name: item.account_display_name.clone(),
            note: String::new(),
            avatar: item.account_avatar_url.clone(),
            header: String::new(),
            bot: false,
            locked: false,
            followers_count: 0,
            following_count: 0,
            statuses_count: 0,
            last_status_at: None,
        },
        favourites_count: u64::try_from(item.favourites_count).unwrap_or(0),
        reblogs_count: u64::try_from(item.reblogs_count).unwrap_or(0),
        replies_count: u64::try_from(item.replies_count).unwrap_or(0),
        favourited: false,
        reblogged: false,
        bookmarked: false,
        reblog: None,
        in_reply_to_id: None,
        emojis: Vec::new(),
        url: Some(format!(
            "https://{}/@{}/{}",
            item.instance_host, item.account_acct, item.status_id
        )),
        sensitive: false,
        spoiler_text: String::new(),
        media_attachments: Vec::new(),
        mentions: Vec::new(),
        tags: Vec::new(),
        poll: None,
        edited_at: None,
        pinned: None,
    }
}

/// The feed's post list: a heading, one refresh verb, the cards, and one more page.
///
/// Same four states as the account list, minus the degradation line: a post feed either
/// arrives or it does not. The cards arrive as a callback because each one writes an
/// impression when it leaves the screen, which needs the client's clock.
#[component]
pub fn PostList(
    /// The posts, already in the order they should be read.
    #[prop(into)]
    items: Signal<Vec<PostItem>>,
    /// The last read's refusal, when it had one.
    #[prop(into)]
    error: Signal<Option<String>>,
    /// Whether a read is already on its way.
    #[prop(into)]
    refreshing: Signal<bool>,
    /// The cursor of the next page, when there is one.
    #[prop(into)]
    cursor: Signal<Option<String>>,
    /// Paints one card.
    row: Callback<PostItem, AnyView>,
    /// Asks for the first page again.
    #[prop(optional)]
    on_refresh: Option<Callback<()>>,
    /// Asks for the page after the given cursor.
    #[prop(optional)]
    on_more: Option<Callback<String>>,
    /// A platform trigger for the next page, rendered below the button.
    #[prop(optional_no_strip)]
    auto_more: Option<ChildrenFn>,
    /// Whether the list paints its own heading, refusal and empty card. A page that owns
    /// those states passes `false` and keeps the same list rhythm.
    #[prop(default = true)]
    show_states: bool,
) -> impl IntoView {
    let row = StoredValue::new(row);
    let refresh_disabled = on_refresh.is_none();
    let more_disabled = on_more.is_none();

    view! {
        <Show when=move || show_states>
        <div class="mb-4 flex items-baseline justify-between gap-4">
            <h2 class="text-sm font-medium opacity-70">"Posts to read"</h2>
            <button
                class="btn btn-ghost btn-sm gap-2"
                disabled=move || refreshing.get() || refresh_disabled
                on:click=move |_| {
                    if let Some(on_refresh) = on_refresh {
                        on_refresh.run(());
                    }
                }
            >
                <Show when=move || refreshing.get()>
                    <span class="loading loading-spinner loading-xs"></span>
                </Show>
                "Refresh"
            </button>
        </div>

        {move || {
            error
                .get()
                .map(|message| {
                    view! {
                        <div class="alert alert-soft alert-error mb-4" role="alert">
                            <span>{message}</span>
                            <button
                                class="btn btn-sm"
                                disabled=refresh_disabled
                                on:click=move |_| {
                                    if let Some(on_refresh) = on_refresh {
                                        on_refresh.run(());
                                    }
                                }
                            >
                                "Retry"
                            </button>
                        </div>
                    }
                })
        }}

        <Show when=move || {
            items.get().is_empty() && !refreshing.get() && error.get().is_none()
        }>
            <EmptyPosts/>
        </Show>
        </Show>

        <ul class="space-y-3 transition-opacity" class:opacity-50=move || refreshing.get()>
            <For each=move || items.get() key=|item| item.item_id let:item>
                {row.with_value(|row| row.run(item))}
            </For>
        </ul>

        {move || {
            cursor
                .get()
                .map(|cursor| {
                    view! {
                        <div class="mt-6">
                            <button
                                class="btn btn-outline btn-sm w-full"
                                disabled=move || refreshing.get() || more_disabled
                                on:click=move |_| {
                                    if let Some(on_more) = on_more {
                                        on_more.run(cursor.clone());
                                    }
                                }
                            >
                                {move || {
                                    if refreshing.get() { "Loading…" } else { "Load more posts" }
                                }}
                            </button>
                        </div>
                    }
                })
        }}
        {auto_more.map(|more| more())}
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
