//! Adding people to a list.
//!
//! A list member must be an account the instance already knows, so the only way in is a
//! search. The panel owns the search box and its results; the roster above reloads through
//! a callback rather than sharing state, because the two have nothing else in common.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::{Account, SearchResults};

use crate::state::alive::Alive;
use crate::state::bridge::actions;

/// A search box that adds the account you pick.
#[component]
pub fn AddPeople(
    list_id: ReadSignal<Option<String>>,
    set_error: WriteSignal<Option<String>>,
    on_add: Callback<()>,
) -> impl IntoView {
    let (query, set_query) = signal(String::new());
    let (candidates, set_candidates) = signal(Vec::<Account>::new());
    let (busy, set_busy) = signal(false);
    let alive = Alive::new();

    let search = {
        move |()| {
            let text = query.get_untracked();
            if text.trim().is_empty() {
                set_candidates.set(Vec::new());
                return;
            }

            set_busy.set(true);
            spawn_local(async move {
                match actions::search(text.trim(), true).await {
                    Ok(SearchResults { accounts, .. }) => {
                        alive.if_alive(|| set_candidates.set(accounts));
                    }
                    Err(message) => alive.if_alive(|| set_error.set(Some(message))),
                }
                alive.if_alive(|| set_busy.set(false));
            });
        }
    };

    let add = move |account_id: String| {
        let Some(id) = list_id.get_untracked() else {
            return;
        };

        set_busy.set(true);
        set_error.set(None);
        spawn_local(async move {
            match actions::set_list_member(&id, &account_id, true).await {
                Ok(()) => {
                    alive.if_alive(|| set_query.set(String::new()));
                    alive.if_alive(|| set_candidates.set(Vec::new()));
                    on_add.run(());
                }
                Err(message) => alive.if_alive(|| set_error.set(Some(message))),
            }
            alive.if_alive(|| set_busy.set(false));
        });
    };

    view! {
        <form
            class="flex gap-2"
            on:submit=move |event| {
                event.prevent_default();
                search(());
            }
        >
            <input
                class="input input-bordered input-sm w-full"
                placeholder="Add people by name or @acct"
                prop:value=move || query.get()
                on:input=move |event| set_query.set(event_target_value(&event))
            />
            <button class="btn btn-secondary btn-sm" type="submit" disabled=move || busy.get()>
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
                                disabled=move || busy.get()
                                on:click=move |_| add(id.clone())
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
