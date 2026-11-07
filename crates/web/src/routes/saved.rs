//! Saved statuses: the two shelves the instance keeps for the signed-in account.
//!
//! Favourites and bookmarks are separate endpoints with the same shape, so they share a
//! screen with a tab each. Both pages are cursored the same way the timelines are: only a
//! full page produces a `next_max_id`, and that id is what "Load more" sends back.

use leptos::prelude::*;

use crate::ui::page::PageHeader;
use leptos::task::spawn_local;
use nodus_instance::model::{Page, Status};

use super::status::StatusCard;
use crate::state::alive::Alive;
use crate::state::bridge::actions;

/// Which shelf is showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Shelf {
    Favourites,
    Bookmarks,
}

impl Shelf {
    /// The tab label.
    fn label(self) -> &'static str {
        match self {
            Self::Favourites => "Favourites",
            Self::Bookmarks => "Bookmarks",
        }
    }
}

/// One page of a shelf, or the reason it could not be read.
async fn page(shelf: Shelf, after: Option<&str>) -> Result<Page<Status>, String> {
    match shelf {
        Shelf::Favourites => actions::favourites(after).await,
        Shelf::Bookmarks => actions::bookmarks(after).await,
    }
}

/// The saved screen: a tab per shelf and one list beneath it.
#[component]
pub fn Saved() -> impl IntoView {
    let (shelf, set_shelf) = signal(Shelf::Favourites);
    let (statuses, set_statuses) = signal(Vec::<Status>::new());
    let (cursor, set_cursor) = signal(None::<String>);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);

    let load = move |which: Shelf, after: Option<String>| {
        let alive = Alive::new();
        set_loading.set(true);
        set_error.set(None);
        spawn_local(async move {
            match page(which, after.as_deref()).await {
                Ok(page) => {
                    alive.if_alive(|| set_cursor.set(page.next_max_id));
                    // Collect first: filtering against the same list we are extending
                    // would borrow it twice.
                    let fresh: Vec<Status> = page
                        .items
                        .into_iter()
                        .filter(|status| {
                            !statuses
                                .get_untracked()
                                .iter()
                                .any(|seen| seen.id == status.id)
                        })
                        .collect();
                    alive.if_alive(|| set_statuses.update(|list| list.extend(fresh)));
                }
                Err(message) => alive.if_alive(|| set_error.set(Some(message))),
            }
            alive.if_alive(|| set_loading.set(false));
        });
    };

    // The first page only: switching tabs clears the list instead of merging, because a
    // favourite is not a bookmark and one list holding both would be a lie about which.
    Effect::new(move |_| load(Shelf::Favourites, None));

    let more = move || {
        if let Some(after) = cursor.get_untracked() {
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
                                set_statuses.set(Vec::new());
                                set_cursor.set(None);
                                load(which, None);
                            }
                        }
                    >
                        {which.label()}
                    </button>
                </For>
            </div>

            <Show when=move || {
                !loading.get() && statuses.get().is_empty() && error.get().is_none()
            }>
                <p class="text-sm opacity-60">
                    {move || match shelf.get() {
                        Shelf::Favourites => "Nothing is favourited yet.",
                        Shelf::Bookmarks => "Nothing is bookmarked yet.",
                    }}
                </p>
            </Show>

            <ul class="flex flex-col gap-3">
                <For each=move || statuses.get() key=|status| status.id.clone() let:status>
                    <StatusCard status=status/>
                </For>
            </ul>

            <Show when=move || cursor.get().is_some()>
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
