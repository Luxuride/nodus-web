//! Adding people to a list.
//!
//! The panel is `nodus_view::pages::lists::AddPeople`: the search box, the candidates and the
//! Add buttons. What stays here is the search itself and the membership write, because the
//! draft and the candidates are one search's answer and belong to the screen that owns the
//! transport.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::{Account, SearchResults};
use nodus_view::pages::lists::AddPeople as AddPeoplePanel;

use crate::state::alive::Alive;
use crate::state::bridge::actions;

/// A search box that adds the account you pick.
#[component]
pub fn AddPeople(
    list_id: Signal<Option<String>>,
    set_error: WriteSignal<Option<String>>,
    on_add: Callback<()>,
) -> impl IntoView {
    let (query, set_query) = signal(String::new());
    let (candidates, set_candidates) = signal(Vec::<Account>::new());
    let (busy, set_busy) = signal(false);
    let alive = Alive::new();

    let on_query = Callback::new(move |text: String| set_query.set(text));

    let search = Callback::new(move |()| {
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
    });

    let add = Callback::new(move |account_id: String| {
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
    });

    view! {
        <AddPeoplePanel
            query=query.into()
            candidates=candidates.into()
            busy=busy.into()
            on_query=on_query
            on_search=search
            on_add=add
        />
    }
}
