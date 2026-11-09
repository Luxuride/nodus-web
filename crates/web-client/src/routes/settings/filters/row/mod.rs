//! One filter, with the only control the card offers: delete it.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::{Filter, FilterAction};

use crate::state::bridge::actions;

/// A filter row.
///
/// Takes the reload callback rather than a signal so the card owns the list: a row that
/// kept its own copy would have to reconcile two answers to "what filters exist".
#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "the row renders the filter it is handed and never keeps it"
)]
pub fn FilterRow(filter: Filter, reload: Callback<()>) -> impl IntoView {
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
    let id = filter.id.clone();

    view! {
        <li class="flex items-start justify-between gap-2 rounded-box bg-base-200 px-3 py-2">
            <div class="min-w-0">
                <p class="truncate text-sm font-medium">{filter.title.clone()}</p>
                <p class="truncate text-xs opacity-70">{format!("{action}: {keywords}")}</p>
            </div>
            <button
                class="btn btn-ghost btn-xs"
                on:click=move |_| {
                    let id = id.clone();
                    spawn_local(async move {
                        // A delete that failed leaves the row in place, which is the
                        // honest state: the filter is still on the server.
                        if actions::delete_filter(&id).await.is_ok() {
                            reload.run(());
                        }
                    });
                }
            >
                "Delete"
            </button>
        </li>
    }
}
