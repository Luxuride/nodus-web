//! The whole status card, shared by the server paint and the client mount.
//!
//! A boost is unwrapped here rather than at the call site, because the wrapper carries an
//! empty body: the line above the card names who boosted, and everything below it is the
//! boosted status read from `reblog`.
//!
//! The card is one box of four parts — the header, the body, the action row and whatever the
//! client adds under it — so a paint cannot assemble three of them and forget the fourth.
//! What separates the paints is the press target around the body: on the client a press opens
//! the post unless it landed on something that owns its own click, and knowing that is
//! `web-sys`. The client therefore supplies its own wrapped body through `body`, and the
//! wrapper this component draws is the server's, where there is nothing to press.

use leptos::prelude::*;
use nodus_instance::model::Status;

use crate::media_grid::Picture;
use crate::status_body::{StatusBody, StatusBodyProps};
use crate::status_footer::{StatusFooter, StatusFooterProps};
use crate::status_header::{StatusHeader, StatusHeaderProps};

/// A status card: who wrote it, what it says, and what can be done with it.
///
/// Every callback takes the status id; on the server all of them are absent, so the verbs
/// keep their boxes and drop only their affordance.
#[component]
pub fn StatusCard(
    /// The status to draw, boost wrapper or not.
    status: Status,
    /// How old the post is, already formatted.
    age: String,
    /// Where the author's profile lives.
    author_path: String,
    /// Where the thread lives.
    thread_path: String,
    /// Follows an address in the header.
    #[prop(optional)]
    on_navigate: Option<Callback<String>>,
    /// Opens the post from the body.
    #[prop(optional)]
    on_open: Option<Callback<String>>,
    /// Boosts, or undoes the boost.
    #[prop(optional)]
    on_boost: Option<Callback<String>>,
    /// Favourites, or removes the favourite.
    #[prop(optional)]
    on_favourite: Option<Callback<String>>,
    /// Bookmarks, or removes the bookmark.
    #[prop(optional)]
    on_bookmark: Option<Callback<String>>,
    /// Opens the thread from the action row.
    #[prop(optional)]
    on_reply: Option<Callback<String>>,
    /// The client's own controls, drawn where the menu sits.
    #[prop(optional)]
    before_verbs: Option<ChildrenFn>,
    /// The client's own controls, drawn after the verbs.
    #[prop(optional)]
    after_verbs: Option<ChildrenFn>,
    /// Opens the viewer on one picture; absent where nothing can be pressed.
    #[prop(optional)]
    on_open_media: Option<Callback<(Vec<Picture>, usize)>>,
    /// Sends the picked options of a poll; absent where there is nowhere to send them.
    #[prop(optional)]
    on_vote: Option<Callback<(String, Vec<usize>)>>,
    /// Whether media offers a player or a gallery; false for the server paint.
    #[prop(default = true)]
    interactive: bool,
    /// The client's body, already wrapped in the press target it needs.
    #[prop(optional)]
    body: Option<ChildrenFn>,
    /// The client's own addition under the action row, such as its transport error.
    #[prop(optional)]
    extra: Option<ChildrenFn>,
) -> impl IntoView {
    // The line and the unwrapping are one decision: a boost shows the boosted status, so the
    // wrapper contributes its author's name and nothing else.
    let booster = status.reblog.as_ref().map(|_| status.account.acct.clone());
    let inner = match status.reblog {
        Some(reblog) => *reblog,
        None => status,
    };
    let id = StoredValue::new(inner.id.clone());
    let edited = inner.edited_at.is_some();
    // Stored for the same reason `NotificationCard` stores its slots: the body of the card is
    // built once, and a captured `ChildrenFn` would have to be cloned out of a `Fn` closure.
    let body = StoredValue::new(body);
    let extra = StoredValue::new(extra);

    view! {
        <li class="card border border-base-300 bg-base-100 shadow-sm">
            <div class="card-body gap-3 p-4">
                {booster.as_ref().map(|account| view! {
                    <p class="text-xs opacity-60">{format!("{account} boosted")}</p>
                })}
                {StatusHeader(StatusHeaderProps {
                    status: inner.clone(),
                    age,
                    author_path,
                    thread_path,
                    on_navigate,
                })}

                // The whole body opens the post: reading it in place is the point of a
                // timeline, and reaching for the timestamp to do it is a second step nobody
                // wants.
                {body.with_value(|body| {
                    body.as_ref().map_or_else(
                        || {
                            view! {
                                <div class="cursor-pointer" on:click=move |_| {
                                    if let Some(on_open) = on_open {
                                        on_open.run(id.get_value());
                                    }
                                }>
                                    {StatusBody(StatusBodyProps {
                                        status: inner.clone(),
                                        on_open_media,
                                        on_vote,
                                        interactive,
                                        revealed_init: false,
                                    })}
                                </div>
                            }
                            .into_any()
                        },
                        |children| children(),
                    )
                })}

                {StatusFooter(StatusFooterProps {
                    status: inner,
                    edited,
                    on_reply,
                    on_boost,
                    on_favourite,
                    on_bookmark,
                    before_verbs,
                    after_verbs,
                })}
                {extra.with_value(|extra| extra.as_ref().map(|children| children()))}
            </div>
        </li>
    }
}
