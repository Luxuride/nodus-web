//! Adding people to the selected list: a search, and the candidates it answers with.
//!
//! The draft and the candidates are the caller's — they are one search's answer, not the
//! screen's state — so the form takes what was typed and two verbs, and the panel draws the
//! same boxes with nothing behind them when it is painted without a transport.

use leptos::prelude::*;
use nodus_client_core::model::Account;

use super::stop_submit;

/// The search that adds people to a list.
#[component]
pub fn AddPeople(
    /// What the reader has typed into the search.
    query: Signal<String>,
    /// The accounts the last search answered with.
    candidates: Signal<Vec<Account>>,
    /// Whether a write is on its way.
    busy: Signal<bool>,
    /// What the reader typed into the search.
    #[prop(optional)]
    on_query: Option<Callback<String>>,
    /// Searches for whoever matches the query.
    #[prop(optional)]
    on_search: Option<Callback<()>>,
    /// Adds one account to the list.
    #[prop(optional)]
    on_add: Option<Callback<String>>,
) -> impl IntoView {
    view! {
        <form
            class="flex gap-2"
            on:submit=move |event| {
                stop_submit(&event);
                if let Some(on_search) = on_search {
                    on_search.run(());
                }
            }
        >
            <input
                class="input input-bordered input-sm w-full"
                placeholder="Add people by name or @acct"
                prop:value=move || query.get()
                disabled=on_query.is_none()
                on:input=move |event| {
                    if let Some(on_query) = on_query {
                        on_query.run(event_target_value(&event));
                    }
                }
            />
            <button
                class="btn btn-secondary btn-sm"
                type="submit"
                disabled=move || busy.get() || on_search.is_none()
            >
                "Search"
            </button>
        </form>
        <ul class="flex flex-col gap-1">
            <For each=move || candidates.get() key=|account| account.id.clone() let:account>
                {
                    let id = account.id.clone();
                    view! {
                        <li class="flex items-center justify-between gap-2 text-sm">
                            <span class="truncate">
                                {format!("@{} · {}", account.acct, account.display_name)}
                            </span>
                            <button
                                class="btn btn-ghost btn-xs"
                                disabled=move || busy.get() || on_add.is_none()
                                on:click=move |_| {
                                    if let Some(on_add) = on_add {
                                        on_add.run(id.clone());
                                    }
                                }
                            >
                                "Add"
                            </button>
                        </li>
                    }
                }
            </For>
        </ul>
    }
}
