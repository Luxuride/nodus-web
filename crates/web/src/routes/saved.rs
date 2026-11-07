//! Saved statuses: the two shelves the instance keeps for the signed-in account.
//!
//! Favourites and bookmarks are separate endpoints with the same shape, so they share a
//! screen with a tab each. Both pages are cursored the same way the timelines are: only a
//! full page produces a `next_max_id`, and that id is what "Load more" sends back.

use leptos::prelude::*;

use crate::ui::page::PageHeader;
use leptos::task::spawn_local;
use super::status::StatusCard;
use crate::state::alive::Alive;
use crate::state::bridge::{pages, revision};
use nodus_sdk::page::{PageItems, PageKind};

/// Which shelf is showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Shelf {
    Favourites,
    Bookmarks,
}

impl Shelf {
    /// The page this shelf reads from SDK state.
    fn kind(self) -> PageKind {
        match self {
            Self::Favourites => PageKind::Favourites,
            Self::Bookmarks => PageKind::Bookmarks,
        }
    }

    /// The tab label.
    fn label(self) -> &'static str {
        match self {
            Self::Favourites => "Favourites",
            Self::Bookmarks => "Bookmarks",
        }
    }
}

/// The saved screen: a tab per shelf and one list beneath it.
#[component]
pub fn Saved() -> impl IntoView {
    let revision = revision();
    let (shelf, set_shelf) = signal(Shelf::Favourites);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);

    // The shelf is read from the SDK: two shelves live there side by side, so switching
    // tabs is a read, not a fetch.
    let held = move || {
        revision.get();
        pages::items(&shelf.get().kind(), |items| match items {
            PageItems::Statuses(list) => Some(list),
            _ => None,
        })
    };
    let statuses = move || held().map(|(list, _)| list).unwrap_or_default();
    let cursor = move || held().and_then(|(_, cursor)| cursor);

    let load = move |which: Shelf, after: Option<String>| {
        let alive = Alive::new();
        set_loading.set(true);
        set_error.set(None);
        spawn_local(async move {
            if let Err(message) = pages::ensure_after(which.kind(), after).await {
                alive.if_alive(|| set_error.set(Some(message)));
            }
            alive.if_alive(|| set_loading.set(false));
        });
    };

    // The first page only: switching tabs clears the list instead of merging, because a
    // favourite is not a bookmark and one list holding both would be a lie about which.
    Effect::new(move |_| load(Shelf::Favourites, None));

    let more = move || {
        if let Some(after) = cursor() {
            load(shelf.get_untracked(), Some(after));
        }
    };

    view! {
        <section class="flex flex-col gap-3">
            <PageHeader title="Saved" description="Your favourites and bookmarks, newest first."/>
            <div role="tablist" class="tabs tabs-box w-fit">
                <For
                    each=|| [Shelf::Favourites, Shelf::Bookmarks]
                    key=|which| which.label()
                    let:which
                >
                    <button
                        role="tab"
                        class="tab"
                        class:tab-active=move || shelf.get() == which
                        on:click=move |_| {
                            if shelf.get_untracked() != which {
                                set_shelf.set(which);
                                // The SDK may already hold this shelf; `ensure` decides
                                // whether the network is needed, so no list is cleared.
                                load(which, None);
                            }
                        }
                    >
                        {which.label()}
                    </button>
                </For>
            </div>

            <Show when=move || !loading.get() && statuses().is_empty() && error.get().is_none()>
                <p class="text-sm opacity-60">
                    {move || match shelf.get() {
                        Shelf::Favourites => "Nothing is favourited yet.",
                        Shelf::Bookmarks => "Nothing is bookmarked yet.",
                    }}
                </p>
            </Show>

            <ul class="flex flex-col gap-3">
                <For each=statuses key=|status| status.id.clone() let:status>
                    <StatusCard status=status/>
                </For>
            </ul>

            <Show when=move || cursor().is_some()>
                <button class="btn btn-ghost btn-sm w-full" disabled=move || loading.get() on:click=move |_| more()>
                    "Load more"
                </button>
            </Show>

            <Show when=move || error.get().is_some()>
                <p class="text-error text-xs">{move || error.get().unwrap_or_default()}</p>
            </Show>
        </section>
    }
}
