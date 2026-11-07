//! The home timeline, with the public ones beside it.
//!
//! The SDK owns the cursor; this view owns the statuses it rendered. They are two
//! different questions — "where do I resume" versus "what is on screen" — and a failed
//! fetch must not lose the cursor while a successful one must not duplicate a status.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::Status;

use crate::routes::status::StatusCard;
use crate::state::alive::Alive;
use crate::state::bridge::actions;

/// Which timeline a tab shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Home,
    Local,
    Federated,
}

impl Tab {
    fn label(self) -> &'static str {
        match self {
            Self::Home => "Home",
            Self::Local => "Local",
            Self::Federated => "Federated",
        }
    }
}

/// A timeline page and the cursor that continues it.
///
/// `Clone` because a signal hands out a copy, and the clone is what makes the view
/// independent of the buffer the next page will append to.
#[derive(Debug, Clone, Default)]
struct Timelines {
    statuses: Vec<Status>,
    cursor: Option<String>,
}

/// The home timeline.
#[component]
pub fn Home() -> impl IntoView {
    let (tab, set_tab) = signal(Tab::Home);
    let (timelines, set_timelines) = signal(Timelines::default());
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(None::<String>);
    let alive = Alive::new();

    let load = move |kind: Tab, after: Option<String>| {
        set_loading.set(true);
        set_error.set(None);
        spawn_local(async move {
            let appending = after.is_some();

            let page = match (kind, after) {
                (Tab::Home, cursor) => actions::home_page(cursor.as_deref()).await,
                (Tab::Local, cursor) => actions::public_page(cursor.as_deref(), true).await,
                (Tab::Federated, cursor) => actions::public_page(cursor.as_deref(), false).await,
            };
            match page {
                Ok(page) => alive.if_alive(|| {
                    set_timelines.update(|timelines| {
                        // A new tab starts from nothing; "load more" appends. The cursor is
                        // the only difference, and the caller knows which it asked for.
                        if !appending {
                            timelines.statuses.clear();
                        }
                        timelines
                            .statuses
                            .extend(dedupe(page.items, &timelines.statuses));
                        timelines.cursor = page.next_max_id;
                    });
                }),
                Err(message) => alive.if_alive(|| set_error.set(Some(message))),
            }
            alive.if_alive(|| set_loading.set(false));
        });
    };

    // The first page, once; a later read of the same tab must not refetch what is
    // already on screen.
    Effect::new(move |_| {
        let kind = tab.get();
        let empty = timelines.with(|timelines| timelines.statuses.is_empty());
        if empty {
            load(kind, None);
        }
    });

    view! {
        <section class="flex flex-col gap-3">
            <div role="tablist" class="tabs tabs-box w-fit">
                <For each=|| [Tab::Home, Tab::Local, Tab::Federated] key=|tab| format!("{tab:?}") let:kind>
                    <button
                        role="tab"
                        class="tab"
                        class:tab-active=move || tab.get() == kind
                        on:click=move |_| {
                            set_timelines.set(Timelines::default());
                            set_tab.set(kind);
                            load(kind, None);
                        }
                    >
                        {kind.label()}
                    </button>
                </For>
            </div>

            <Show when=move || loading.get() && timelines.get().statuses.is_empty()>
                <p class="text-sm opacity-60">"Loading…"</p>
            </Show>

            <ul class="flex flex-col gap-3">
                <For
                    each=move || timelines.get().statuses
                    key=|status| status.id.clone()
                    let:status
                >
                    <StatusCard status=status/>
                </For>
            </ul>

            <Show when=move || !timelines.get().statuses.is_empty()>
                <button
                    class="btn btn-ghost btn-sm w-full"
                    disabled=move || loading.get() || timelines.get().cursor.is_none()
                    on:click=move |_| {
                        let kind = tab.get_untracked();
                        let cursor = timelines.get_untracked().cursor;
                        load(kind, cursor);
                    }
                >
                    {move || {
                        if loading.get() { "Loading…" } else { "Load more" }
                    }}
                </button>
            </Show>

            <Show when=move || error.get().is_some()>
                <p class="text-error text-xs">{move || error.get().unwrap_or_default()}</p>
            </Show>
        </section>
    }
}

/// Drops statuses already on screen, keeping the incoming order.
///
/// Instances overlap pages when something new arrives between two requests, and a
/// duplicate key in a keyed list is a Leptos panic, not a cosmetic bug.
fn dedupe(incoming: Vec<Status>, existing: &[Status]) -> Vec<Status> {
    incoming
        .into_iter()
        .filter(|status| !existing.iter().any(|seen| seen.id == status.id))
        .collect()
}
