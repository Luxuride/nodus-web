//! The selected list's timeline.
//!
//! Same shape as the home timeline: one page, a cursor, "Load more". The only difference
//! is where the page comes from, which is why this panel does not reuse `Home` — that one
//! owns a tab that switches between three endpoints.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_sdk::Timeline;

use super::super::status::StatusCard;
use crate::state::alive::Alive;
use crate::state::bridge::{revision, timeline};

/// A list timeline, or nothing when no list is selected.
#[component]
pub fn ListTimeline(list_id: ReadSignal<Option<String>>) -> impl IntoView {
    let changed = revision();
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(None::<String>);

    // The page lives in the SDK, like every other timeline: this panel keeps no statuses
    // of its own, so selecting the list again paints from state instead of refetching.
    let which = move || list_id.get().map(Timeline::List);
    let statuses = move || {
        changed.get();
        which()
            .map(|which| timeline::read(which, None).0)
            .unwrap_or_default()
    };
    let cursor = move || {
        changed.get();
        which().and_then(|which| timeline::read(which, None).1)
    };

    let load = move |id: String, after: Option<String>| {
        let alive = Alive::new();
        set_loading.set(true);
        set_error.set(None);
        spawn_local(async move {
            match timeline::ensure(Timeline::List(id), None, after).await {
                Ok(_) => {}
                Err(message) => alive.if_alive(|| set_error.set(Some(message))),
            }
            alive.if_alive(|| set_loading.set(false));
        });
    };

    // Selecting another list must not append to the previous one's page: the cursor
    // belongs to a list, so it is dropped with the contents.
    Effect::new(move |_| {
        if let Some(id) = list_id.get() {
            load(id, None);
        }
    });

    let more = move || {
        if let (Some(id), Some(after)) = (list_id.get_untracked(), cursor()) {
            load(id, Some(after));
        }
    };

    view! {
        <Show when=move || list_id.get().is_some()>
            <ul class="flex flex-col gap-3">
                <For each=move || statuses() key=|status| status.id.clone() let:status>
                    <StatusCard status=status/>
                </For>
            </ul>

            <Show when=move || {
                !loading.get() && statuses().is_empty() && error.get().is_none()
            }>
                <p class="text-sm opacity-60">"Nobody on this list has posted yet."</p>
            </Show>

            <Show when=move || cursor().is_some()>
                <button
                    class="btn btn-ghost btn-sm w-full"
                    disabled=move || loading.get()
                    on:click=move |_| more()
                >
                    "Load more"
                </button>
            </Show>

            <Show when=move || error.get().is_some()>
                <p class="text-error text-xs">{move || error.get().unwrap_or_default()}</p>
            </Show>
        </Show>
    }
}
