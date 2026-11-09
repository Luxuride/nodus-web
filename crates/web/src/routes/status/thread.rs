//! One status and its conversation: ancestors, the status, the replies, a reply box.
//!
//! The thread is loaded in two calls — the status itself and its context — because
//! Mastodon splits them and because the status is worth showing even when the context
//! endpoint is slow or fails.
//!
//! The page is `nodus_view`'s `ThreadPage`, the same tree the server paints. What stays
//! here is what the page may not know: the two reads, the filters (a thread assembled from
//! cache never passes through the server's eye again, so the matching rule has to run
//! here), the clock, and the composer's transport.

use std::sync::Arc;

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::FilterContext;
use nodus_instance::model::Status;
use nodus_view::components::CardRenderer;

use crate::routes::filtering::Filtered;
use crate::routes::format::relative_time;
use crate::state::alive::Alive;
use crate::state::bridge::actions;
use crate::state::bridge::timeline;

use super::reply::ReplyBox;

/// A thread, with a composer at the bottom.
#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "a Leptos component receives its props by value"
)]
pub fn Thread(id: String) -> impl IntoView {
    let (status, set_status) = signal(None::<Status>);
    let (ancestors, set_ancestors) = signal(Vec::<Status>::new());
    let (replies, set_replies) = signal(Vec::<Status>::new());
    let (error, set_error) = signal(None::<String>);
    let alive = Alive::new();

    let root = id.clone();
    Effect::new(move |_| {
        let for_status = root.clone();

        spawn_local(async move {
            match actions::status(&for_status).await {
                Ok(found) => {
                    // The thread's status becomes SDK state like any other, so a card
                    // that shows it elsewhere is already correct.
                    timeline::upsert(vec![found.clone()]);
                    alive.if_alive(|| set_status.set(Some(found)));
                }
                Err(message) => alive.if_alive(|| set_error.set(Some(message))),
            }
        });
        let for_thread = root.clone();

        spawn_local(async move {
            match actions::thread(&for_thread).await {
                Ok(context) => {
                    let mut all = context.ancestors.clone();
                    all.extend(context.descendants.clone());
                    timeline::upsert(all);
                    alive.if_alive(|| {
                        set_ancestors.set(context.ancestors);
                        set_replies.set(context.descendants);
                    });
                }
                // The status is already on screen; a failed context is a note, not a
                // blank page.
                Err(message) => alive.if_alive(|| set_error.set(Some(message))),
            }
        });
    });

    // Every status goes through `Filtered`, which paints the client's card and hides what the
    // reader asked not to see; the page never decides what a post looks like.
    let card: CardRenderer = Arc::new(|status: Status| {
        view! { <Filtered status=status context=FilterContext::Thread/> }.into_any()
    });
    let age_of: Callback<String, String> =
        Callback::new(|created_at: String| relative_time(&created_at));

    // The composer is transport, so it is a slot the page paints at its foot. The read's own
    // refusals sit beside it: they are the only line the server never shows, and the column
    // that holds them is the one the page draws.
    let parent = id.clone();
    let reply_slot: ChildrenFn = Arc::new(move || {
        view! {
            <ReplyBox parent=parent.clone() set_replies=set_replies set_error=set_error/>
            <Show when=move || error.get().is_some()>
                <p class="text-error text-xs">{move || error.get().unwrap_or_default()}</p>
            </Show>
        }
        .into_any()
    });

    view! {
        {move || match status.get() {
            Some(focused) => view! {
                <nodus_view::thread::ThreadPage
                    ancestors=ancestors.get()
                    focused=focused
                    replies=replies
                    card=card.clone()
                    age_of=age_of
                    reply_slot=reply_slot.clone()
                />
            }
            .into_any(),
            // Nothing has arrived yet: the column keeps its width and says why, if the read
            // refused before the status was in hand.
            None => view! {
                <nodus_view::Screen>
                    <Show when=move || error.get().is_some()>
                        <p class="text-error text-xs">{move || error.get().unwrap_or_default()}</p>
                    </Show>
                </nodus_view::Screen>
            }
            .into_any(),
        }}
    }
}
