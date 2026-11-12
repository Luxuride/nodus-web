//! The reader's filters: phrases that hide a post, or put it behind a warning.

use std::sync::Arc;

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::FilterAction;
use nodus_pages::pages::settings::lists::{FilterItem, FiltersCard as SharedFiltersCard};
use nodus_sdk::page::{PageItems, PageKind};

mod new;

use self::new::NewFilterForm;

use crate::state::alive::Alive;
use crate::state::bridge::{actions, pages, revision};

/// Settings card listing filters and offering to add one.
///
/// The server applies filters to timelines it returns; this screen is the editor, so the
/// list here is the account's own record rather than a local guess.
#[component]
pub fn FiltersCard() -> impl IntoView {
    let alive = Alive::new();
    let revision = revision();
    // Read out of the SDK's page store: this card is an editor, not a cache owner, so a
    // second visit paints the filters it already has instead of asking again.
    let filters = Signal::derive(move || {
        revision.get();
        pages::items(&PageKind::Filters, |items| match items {
            PageItems::Filters(list) => Some(list),
            _ => None,
        })
        .map(|(list, _)| list)
        .unwrap_or_default()
        .into_iter()
        .map(|filter| {
            let keywords = filter
                .keywords
                .iter()
                .map(|keyword| keyword.keyword.clone())
                .collect::<Vec<_>>()
                .join(", ");
            let action = match filter.filter_action {
                FilterAction::Hide => "hides",
                FilterAction::Warn => "warns",
            };
            FilterItem {
                id: filter.id,
                title: filter.title,
                detail: format!("{action}: {keywords}"),
            }
        })
        .collect()
    });
    let (busy, set_busy) = signal(true);
    let (error, set_error) = signal(None::<String>);

    // After a create or a delete the local copy is known to be wrong, which is the one
    // case that ignores the store's timestamp; on mount it may be trusted.
    let reload = Callback::new(move |()| {
        spawn_local(async move {
            let outcome = pages::refresh(PageKind::Filters).await;
            alive.if_alive(|| {
                if let Err(message) = outcome {
                    set_error.set(Some(message));
                }
                set_busy.set(false);
            });
        });
    });

    Effect::new(move |_| {
        spawn_local(async move {
            if let Err(message) = pages::ensure(PageKind::Filters).await {
                alive.if_alive(|| set_error.set(Some(message)));
            }
            alive.if_alive(|| set_busy.set(false));
        });
    });

    let delete = Callback::new(move |id: String| {
        spawn_local(async move {
            // A delete that failed leaves the row in place, which is the honest state:
            // the filter is still on the server.
            if actions::delete_filter(&id).await.is_ok() {
                reload.run(());
            }
        });
    });

    let add_form: ChildrenFn = Arc::new(move || {
        view! {
            <NewFilterForm on_created=Callback::new(move |()| reload.run(())) set_error=set_error/>
        }
        .into_any()
    });

    view! {
        <SharedFiltersCard
            filters=filters
            busy=Signal::derive(move || busy.get())
            error=Signal::derive(move || error.get())
            on_delete=Some(delete)
            add_form=add_form
        />
    }
}
