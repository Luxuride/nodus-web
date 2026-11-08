//! One status and its conversation: ancestors, the status, the replies, a reply box.
//!
//! The thread is loaded in two calls — the status itself and its context — because
//! Mastodon splits them and because the status is worth showing even when the context
//! endpoint is slow or fails.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::FilterContext;
use nodus_instance::model::Status;

use crate::routes::filtering::Filtered;
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

    let parent = id.clone();

    view! {
        <section class="flex flex-col gap-3">
            <Show when=move || !ancestors.get().is_empty()>
                <ul class="flex flex-col gap-3">
                    <For each=move || ancestors.get() key=|status| status.id.clone() let:status>
                        <Filtered status=status context=FilterContext::Thread/>
                    </For>
                </ul>
            </Show>

            <Show when=move || status.get().is_some()>
                <ul class="flex flex-col gap-3">
                    {move || status.get().map(|status| view! { <Filtered status=status context=FilterContext::Thread/> })}
                </ul>
            </Show>

            <Show when=move || !replies.get().is_empty()>
                <h2 class="mt-2 text-sm font-semibold opacity-70">
                    {move || format!("{} replies", replies.get().len())}
                </h2>
                <ul class="flex flex-col gap-3">
                    <For each=move || replies.get() key=|status| status.id.clone() let:status>
                        <Filtered status=status context=FilterContext::Thread/>
                    </For>
                </ul>
            </Show>

            <ReplyBox parent=parent set_replies=set_replies set_error=set_error/>
            <Show when=move || error.get().is_some()>
                <p class="text-error text-xs">{move || error.get().unwrap_or_default()}</p>
            </Show>
        </section>
    }
}
