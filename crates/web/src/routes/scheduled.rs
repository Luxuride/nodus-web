//! Posts queued for later.
//!
//! This screen is the queue, not a way to add to it: a post is scheduled from the box
//! that also holds its text, so there is one place to write and one place to look at what
//! is waiting. Everything here can do is read the queue and cancel an entry.

mod row;

use leptos::prelude::*;

use self::row::QueuedRow;
use crate::state::alive::Alive;
use crate::state::bridge::{actions, dispatch, pages, revision};
use crate::ui::page::PageHeader;
use leptos::task::spawn_local;
use nodus_sdk::page::{PageItems, PageKind};

/// The queue of posts waiting to be published.
#[component]
pub fn Scheduled() -> impl IntoView {
    let revision = revision();
    let (loading, set_loading) = signal(!pages::held(&PageKind::Scheduled));
    let (error, set_error) = signal(None::<String>);
    let alive = Alive::new();

    let items = move || {
        revision.get();
        pages::items(&PageKind::Scheduled, |items| match items {
            PageItems::Scheduled(list) => Some(list),
            _ => None,
        })
        .map(|(list, _)| list)
        .unwrap_or_default()
    };

    let reload = move || {
        let alive = alive;
        spawn_local(async move {
            // A queue that changed on another device must be re-read, so the page is
            // marked stale before asking rather than trusted.
            dispatch(nodus_sdk::Command::PageStale {
                kind: PageKind::Scheduled,
            });
            match pages::ensure(PageKind::Scheduled).await {
                Ok(_) => alive.if_alive(|| set_loading.set(false)),
                Err(message) => alive.if_alive(|| {
                    set_error.set(Some(message));
                    set_loading.set(false);
                }),
            }
        });
    };

    Effect::new(move |_| reload());

    let cancel = Callback::new(move |id: String| {
        spawn_local(async move {
            match actions::cancel_scheduled(&id).await {
                Ok(()) => reload(),
                Err(message) => alive.if_alive(|| set_error.set(Some(message))),
            }
        });
    });

    view! {
        <section class="flex flex-col gap-3">
            <PageHeader
                title="Scheduled"
                description="Posts waiting to be published. Write one from the box above the home timeline."
            />

            {move || error.get().map(|message| view! {
                <div class="alert alert-error py-2 text-sm"><span>{message}</span></div>
            })}

            <Show when=move || loading.get() && items().is_empty()>
                <p class="text-sm opacity-60">"Loading the queue…"</p>
            </Show>

            <ul class="flex flex-col divide-y divide-base-300 rounded-box bg-base-100">
                <For each=items key=|item| item.id.clone() let:item>
                    <QueuedRow item=item on_cancel=cancel/>
                </For>
            </ul>

            <Show when=move || !loading.get() && items().is_empty()>
                <p class="text-sm opacity-60">"Nothing is waiting."</p>
            </Show>
        </section>
    }
}
