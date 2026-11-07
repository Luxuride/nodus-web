//! One recommended post.

use leptos::prelude::*;
use nodus_shared::PostItem;

use super::text::plain_text;

/// A post card: who wrote it, what it says, and where to read the original.
#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "a Leptos component receives its props by value"
)]
pub fn PostCard(item: PostItem) -> impl IntoView {
    let display_name = if item.account_display_name.trim().is_empty() {
        item.account_acct.clone()
    } else {
        item.account_display_name.clone()
    };
    let avatar_url = item.account_avatar_url.clone();
    let has_avatar = !avatar_url.trim().is_empty();
    let body = plain_text(&item.text);
    let link = format!(
        "https://{}/@{}/{}",
        item.instance_host, item.account_acct, item.status_id
    );
    let published = relative_time(&item.published_at);

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
                        <p class="truncate text-sm opacity-60">{item.account_acct.clone()}</p>
                    </div>
                    <span class="text-xs whitespace-nowrap opacity-50">{published}</span>
                </div>

                <p class="whitespace-pre-line text-sm leading-relaxed">{body}</p>

                <div class="flex items-center justify-between gap-2">
                    <span class="text-xs opacity-60">
                        {format!(
                            "{} boosts · {} favourites · {} replies",
                            item.reblogs_count, item.favourites_count, item.replies_count,
                        )}
                    </span>
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

/// A coarse, honest age: exact timestamps are one click away at the instance.
///
/// Parsing is deliberately shallow: the server sends RFC 3339, the browser clock is
/// the user's, and a wrong-by-seconds age is harmless while a panicking parse is not.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "minutes since a past instant, clamped non-negative, fit a u32 for centuries"
)]
fn relative_time(rfc3339: &str) -> String {
    let stamp = js_sys::Date::parse(rfc3339);
    if !stamp.is_finite() {
        return String::new();
    }
    // A skewed client clock can put a just-published post in the future; "-3m" would
    // be a bug report, "now" would be the truth.
    let elapsed_minutes = (js_sys::Date::now() - stamp) / 60_000.0;
    let minutes = if elapsed_minutes <= 0.0 {
        0
    } else {
        elapsed_minutes as u32
    };
    match minutes {
        0 => "now".to_owned(),
        1..=59 => format!("{minutes}m"),
        60..=1439 => format!("{}h", minutes / 60),
        _ => format!("{}d", minutes / (60 * 24)),
    }
}
