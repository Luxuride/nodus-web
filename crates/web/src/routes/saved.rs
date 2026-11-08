//! Saved statuses: the two shelves the instance keeps for the signed-in account.
//!
//! Favourites and bookmarks are separate endpoints with the same shape, so they share a
//! screen with a tab each. Both pages are cursored the same way the timelines are: only a
//! full page produces a `next_max_id`, and that id is what "Load more" sends back.

use leptos::prelude::*;

use super::status::StatusCard;
use crate::router::Route;
use crate::state::alive::Alive;
use crate::state::bridge::{pages, revision};
use crate::ui::page::PageHeader;
use leptos::task::spawn_local;
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

    /// The address this shelf lives at.
    fn slug(self) -> &'static str {
        match self {
            Self::Favourites => "favourites",
            Self::Bookmarks => "bookmarks",
        }
    }

    /// The shelf a `/saved/{slug}` address names.
    fn of(slug: &str) -> Self {
        match slug {
            "bookmarks" => Self::Bookmarks,
            _ => Self::Favourites,
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
/// The shelf a `/saved/{slug}` address names.
/// A Leptos component receives its props by value; the body only reads it.
#[expect(
    clippy::needless_pass_by_value,
    reason = "a Leptos component receives its props by value"
)]
#[component]
pub fn Saved(shelf: String) -> impl IntoView {
    let revision = revision();
    // The address decides the shelf: `/saved/bookmarks` opens Bookmarks.
    let (shelf, _set_shelf) = signal(Shelf::of(&shelf));
    let (loading, set_loading) =
        signal(!pages::held(&PageKind::Favourites) && !pages::held(&PageKind::Bookmarks));
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
        // A shelf we already hold paints immediately; only an empty one looks busy.
        set_loading.set(!pages::held(&which.kind()));
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
    Effect::new(move |_| load(shelf.get_untracked(), None));

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
                    <a
                        role="tab"
                        class="tab"
                        href=format!("/saved/{}", which.slug())
                        class:tab-active=move || shelf.get() == which
                        on:click=move |event| {
                            crate::router::link(event, Route::Saved(which.slug().to_owned()));
                        }
                    >
                        {which.label()}
                    </a>
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
