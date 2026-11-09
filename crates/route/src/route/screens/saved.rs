//! Saved posts, one shelf at a time.

use std::sync::Arc;

use leptos::prelude::*;
use nodus_state::page::{PageItems, PageKind};

use crate::components::CardRenderer;
use crate::pages::saved::{SavedPage, Shelf};

use super::super::Runtime;

#[component]
pub fn SavedScreen(runtime: Arc<dyn Runtime>, shelf: String, card: CardRenderer) -> impl IntoView {
    let shelf = RwSignal::new(Shelf::of(&shelf));
    let revision = runtime.revision();
    let loading = RwSignal::new(runtime.page(&kind(shelf.get_untracked())).is_none());
    let error = RwSignal::new(None::<String>);
    let alive = crate::Alive::new();

    let items = {
        let runtime = Arc::clone(&runtime);
        move || {
            revision.get();
            runtime
                .page(&kind(shelf.get()))
                .and_then(|page| match page.items {
                    PageItems::Statuses(items) => Some(items),
                    _ => None,
                })
                .unwrap_or_default()
        }
    };
    let cursor = {
        let runtime = Arc::clone(&runtime);
        move || {
            revision.get();
            runtime
                .page(&kind(shelf.get()))
                .and_then(|page| page.cursor)
        }
    };

    let load: Callback<(Shelf, Option<String>)> = Callback::new({
        let runtime = Arc::clone(&runtime);
        move |(which, after): (Shelf, Option<String>)| {
            loading.set(runtime.page(&kind(which)).is_none());
            error.set(None);
            let done = Callback::new(move |result: Result<(), String>| {
                alive.if_alive(|| {
                    loading.set(false);
                    if let Err(message) = result {
                        error.set(Some(message));
                    }
                });
            });
            runtime.load_page(kind(which), after, done);
        }
    });

    {
        let runtime = Arc::clone(&runtime);
        Effect::new(move |_| {
            if runtime.page(&kind(shelf.get_untracked())).is_none() {
                load.run((shelf.get_untracked(), None));
            }
        });
    }

    let on_more =
        Callback::new(move |after: String| load.run((shelf.get_untracked(), Some(after))));
    let on_shelf = Callback::new({
        let runtime = Arc::clone(&runtime);
        move |to: Shelf| {
            shelf.set(to);
            runtime.navigate(format!("/saved/{}", to.slug()));
            load.run((to, None));
        }
    });

    view! {
        <SavedPage
            shelf=shelf.into()
            items=Signal::derive(items)
            loading=loading.into()
            error=error.into()
            cursor=Signal::derive(cursor)
            card=card
            on_shelf=on_shelf
            on_more=on_more
        />
    }
}

fn kind(shelf: Shelf) -> PageKind {
    match shelf {
        Shelf::Favourites => PageKind::Favourites,
        Shelf::Bookmarks => PageKind::Bookmarks,
    }
}
