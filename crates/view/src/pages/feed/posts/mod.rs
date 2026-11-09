//! One recommended post.
//!
//! The body and the age arrive already prepared: the body because the server has the
//! instance's HTML and the client has to strip it, and the age because a clock is not
//! state this crate may read (the same reason `PageHeader` takes its counters).

use leptos::prelude::*;
use nodus_shared::PostItem;

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

    let display_name = crate::components::format::display_name(
        &item.account_display_name,
        &item.account_acct,
    );
    let avatar_url = item.account_avatar_url.clone();
    let has_avatar = !avatar_url.trim().is_empty();
    let handle = item.account_acct.clone();
    let counters = format!(
        "{} boosts · {} favourites · {} replies",
        item.reblogs_count, item.favourites_count, item.replies_count,
    );
    let link = format!(
        "https://{}/@{}/{}",
        item.instance_host, item.account_acct, item.status_id
    );

    view! {
        <li class="card border border-base-300 bg-base-100 shadow-sm">
            <div class="card-body gap-3 p-4">
                <div class="flex items-center gap-3">
                    <Show when=move || has_avatar>
                        <div class="avatar">
                            <div class="w-10 rounded-full bg-base-300">
                                <img
                                    src=avatar_url.clone()
                                    alt=""
                                    loading="lazy"
                                    referrerpolicy="no-referrer"
                                />
                            </div>
                        </div>
                    </Show>
                    <div class="min-w-0 flex-1">
                        <p class="truncate font-semibold leading-tight">{display_name}</p>
                        <p class="truncate text-sm opacity-60">{handle.clone()}</p>
                    </div>
                    <span class="text-xs whitespace-nowrap opacity-50">{age}</span>
                </div>

                <p class="whitespace-pre-line text-sm leading-relaxed">{body}</p>

                <div class="flex items-center justify-between gap-2">
                    <span class="text-xs opacity-60">{counters}</span>
                    <a
                        class="btn btn-ghost btn-sm"
                        href=link
                        rel="nofollow noopener noreferrer"
                        target="_blank"
                        referrerpolicy="no-referrer"
                    >
                        "Open"
                    </a>
                </div>
            </div>
        </li>
    }
}
