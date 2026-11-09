//! Who is on the selected list.
//!
//! The panel is `nodus_view::pages::lists::Members`: the roster, its busy flag and its error
//! line, and the slot where adding people hangs. What stays here is the roster read and the
//! membership write, plus the search the page cannot own because it holds its own draft.

use std::sync::Arc;

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::Account;
use nodus_view::pages::lists::Members as MembersPanel;

use self::add::AddPeople;
use crate::state::alive::Alive;
use crate::state::bridge::actions;

mod add;

/// The membership panel: current members, then a search to add more.
#[component]
pub fn Members(list_id: Signal<Option<String>>) -> impl IntoView {
    let (members, set_members) = signal(Vec::<Account>::new());
    let (busy, set_busy) = signal(false);
    let (error, set_error) = signal(None::<String>);
    // Bumped by the add panel: membership changed elsewhere, so the roster is stale.
    let (refresh, set_refresh) = signal(0_u32);
    let alive = Alive::new();

    let remove = Callback::new(move |account_id: String| {
        let Some(id) = list_id.get_untracked() else {
            return;
        };

        set_busy.set(true);
        set_error.set(None);
        spawn_local(async move {
            match actions::set_list_member(&id, &account_id, false).await {
                // The effect reloads the roster: the removal is already on the list.
                Ok(()) => alive.if_alive(|| set_refresh.update(|count| *count += 1)),
                Err(message) => alive.if_alive(|| set_error.set(Some(message))),
            }
            alive.if_alive(|| set_busy.set(false));
        });
    });

    let reload = {
        move || {
            let Some(id) = list_id.get_untracked() else {
                set_members.set(Vec::new());
                return;
            };

            spawn_local(async move {
                match actions::list_accounts(&id).await {
                    Ok(found) => alive.if_alive(|| set_members.set(found)),
                    Err(message) => alive.if_alive(|| set_error.set(Some(message))),
                }
            });
        }
    };

    // Reload on every selection change, not just on mount: the panel has one prop and it
    // is the only thing that makes the contents stale.
    {
        Effect::new(move |_| {
            list_id.get();
            refresh.get();
            reload();
        });
    }

    let added = Callback::new(move |()| set_refresh.update(|count| *count += 1));
    let add: ChildrenFn = Arc::new(move || {
        view! { <AddPeople list_id=list_id set_error=set_error on_add=added/> }.into_any()
    });

    view! {
        <MembersPanel
            list_id=list_id
            members=members.into()
            busy=busy.into()
            error=error.into()
            on_remove=remove
            add=add
        />
    }
}
