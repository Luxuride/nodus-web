//! The action row under a status, shared by the server paint and the client mount.
//!
//! The counts and the three verb boxes are the same on both paints; only the press differs.
//! On the client every verb arrives as a callback and the row is a set of buttons; on the
//! server no callback arrives, so a verb keeps its box and its count but drops the button
//! affordance (`cursor-default`, no hover) instead of losing its width and shifting the row.
//!
//! The two controls that exist only in the app — the post's menu and the copy-link button —
//! arrive as slots, because this crate may not reach for the transport or the clipboard.

use leptos::prelude::*;
use nodus_instance::model::Status;

use crate::icons::Icon;

/// Counts and the actions a reader can take on a status.
///
/// Each callback receives the status id; on the server every one of them is absent.
#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "a Leptos component receives its props by value"
)]
pub fn StatusFooter(
    /// The status this row belongs to.
    status: Status,
    /// Whether the status was edited after it was posted.
    edited: bool,
    /// Where the post's own page lives, when it is not the local `/status/{id}`.
    #[prop(optional, into)]
    thread_path: Option<String>,
    /// Opens the thread.
    #[prop(optional)]
    on_reply: Option<Callback<String>>,
    /// Boosts, or undoes the boost.
    #[prop(optional)]
    on_boost: Option<Callback<String>>,
    /// Favourites, or removes the favourite.
    #[prop(optional)]
    on_favourite: Option<Callback<String>>,
    /// Bookmarks, or removes the bookmark.
    #[prop(optional)]
    on_bookmark: Option<Callback<String>>,
    /// The client's own controls, drawn where the menu sits.
    #[prop(optional)]
    before_verbs: Option<ChildrenFn>,
    /// The client's own controls, drawn after the verbs.
    #[prop(optional)]
    after_verbs: Option<ChildrenFn>,
) -> impl IntoView {
    // The id is read by four handlers, so it lives in a stored value: a `String` captured by
    // value would make the first handler `FnOnce`, and a stored value is `Copy`. The slots
    // are stored for the same reason `NotificationCard` stores its own.
    let target = StoredValue::new(status.id.clone());
    let thread = thread_path.unwrap_or_else(|| format!("/status/{}", status.id));
    let before = StoredValue::new(before_verbs);
    let after = StoredValue::new(after_verbs);

    view! {
        <footer class="flex flex-wrap items-center justify-between gap-x-2 gap-y-1 border-t border-base-200 pt-2">
            <span class="shrink-0 text-xs opacity-60">{format!("{} replies", status.replies_count)}</span>
            <div class="flex w-full min-w-0 flex-wrap items-center justify-between gap-0.5 sm:w-auto sm:justify-end sm:gap-1">
                <a
                    class="btn btn-ghost btn-sm h-9 min-h-9 px-1 sm:h-8 sm:min-h-8 sm:px-3"
                    href=thread
                    title="Reply to this post"
                    aria-label="Reply to this post"
                    on:click=move |event| follow_reply(&event, target, on_reply)
                >
                    <Icon icon=Icon::Reply/>
                    <span class="ml-1 hidden sm:inline">"Reply"</span>
                </a>
                {edited.then(|| view! { <span class="text-xs opacity-50">"edited"</span> })}
                {before.with_value(|before| before.as_ref().map(|children| children()))}
                {verb(
                    Icon::Repeat,
                    Some(status.reblogs_count),
                    status.reblogged,
                    if status.reblogged {
                        "Undo boost"
                    } else {
                        "Boost to your followers"
                    },
                    on_boost,
                    target,
                )}
                {verb(
                    Icon::Star,
                    Some(status.favourites_count),
                    status.favourited,
                    if status.favourited {
                        "Remove from favourites"
                    } else {
                        "Add to favourites"
                    },
                    on_favourite,
                    target,
                )}
                {verb(
                    Icon::Bookmark,
                    None,
                    status.bookmarked,
                    if status.bookmarked {
                        "Remove from bookmarks"
                    } else {
                        "Save for later"
                    },
                    on_bookmark,
                    target,
                )}
                {after.with_value(|after| after.as_ref().map(|children| children()))}
            </div>
        </footer>
    }
}

/// Runs the reply callback, or lets the browser follow the anchor when there is none.
///
/// Without a callback the `href` is the whole answer, which is the server's case: it emits
/// no listener and the anchor is a real link. The non-wasm build keeps a use of `event` so
/// the listener argument is not reported as unused on the target with no
/// `prevent_default`.
fn follow_reply(
    event: &leptos::ev::MouseEvent,
    id: StoredValue<String>,
    on_reply: Option<Callback<String>>,
) {
    #[cfg(not(target_arch = "wasm32"))]
    let _ = event;
    if let Some(on_reply) = on_reply {
        #[cfg(target_arch = "wasm32")]
        event.prevent_default();
        on_reply.run(id.get_value());
    }
}

/// One verb in the row: a button when the app can run it, the same box as a `span`
/// otherwise.
///
/// `count` is `None` for the bookmark, which the client draws as the bare icon. The class
/// string is the client's, with `btn-active` spelled out when the verb is set, so the
/// active highlight survives the round trip through `to_html`.
fn verb(
    icon: Icon,
    count: Option<u64>,
    active: bool,
    title: &'static str,
    on_click: Option<Callback<String>>,
    id: StoredValue<String>,
) -> AnyView {
    let class = if active {
        "btn btn-ghost btn-sm btn-active h-9 min-h-9 px-1 sm:h-8 sm:min-h-8 sm:px-3"
    } else {
        "btn btn-ghost btn-sm h-9 min-h-9 px-1 sm:h-8 sm:min-h-8 sm:px-3"
    };
    let badge = count.map(|count| view! { <span class="ml-1 tabular-nums">{count}</span> });
    if let Some(on_click) = on_click {
        return view! {
            <button class=class title=title on:click=move |_| on_click.run(id.get_value())>
                <Icon icon=icon/>
                {badge}
            </button>
        }
        .into_any();
    }
    view! {
        <span class=format!("{class} cursor-default pointer-events-none") title=title>
            <Icon icon=icon/>
            {badge}
        </span>
    }
    .into_any()
}
