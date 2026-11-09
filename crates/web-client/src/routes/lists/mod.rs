//! Lists: hand-picked sets of accounts with a timeline each.
//!
//! The screen is `nodus_view::pages::lists`: the heading, the create form, the picker and the
//! two panels of the selected list. What stays here is what that crate may not hold — the
//! read of every list, the selection the panels share, and the transports behind create,
//! rename, delete, add and paging.
//!
//! The roster and the timeline are slots rather than props: each holds reads of its own, and
//! both are keyed to the same selection signal, so picking another list is what makes them
//! stale.

use std::sync::Arc;

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::List;
use nodus_view::pages::lists::ListsPage;

use crate::state::alive::Alive;
use crate::state::bridge::actions;

mod members;
mod row;
mod timeline;

use members::Members;
use row::ListRow;
use timeline::ListTimeline;

/// The lists screen.
#[component]
pub fn Lists() -> impl IntoView {
    let (lists, set_lists) = signal(Vec::<List>::new());
    let (selected, set_selected) = signal(None::<String>);
    let (title, set_title) = signal(String::new());
    let (busy, set_busy) = signal(false);
    let (error, set_error) = signal(None::<String>);
    let alive = Alive::new();

    let reload = {
        move || {
            // Called from a request handler as well, which may land after the screen
            // is gone; reading a disposed signal is the same panic as writing one.
            if !alive.is_alive() {
                return;
            }

            spawn_local(async move {
                match actions::lists().await {
                    Ok(found) => alive.if_alive(|| {
                        // Keep the selection if it survived, otherwise open the first
                        // list: a panel with nothing selected looks like the screen
                        // failed to load.
                        let keep = selected
                            .get_untracked()
                            .filter(|id| found.iter().any(|list| &list.id == id));
                        set_selected
                            .set(keep.or_else(|| found.first().map(|list| list.id.clone())));
                        set_lists.set(found);
                    }),
                    Err(message) => alive.if_alive(|| set_error.set(Some(message))),
                }
            });
        }
    };

    Effect::new(move |_| reload());

    let create = move |()| {
        let name = title.get_untracked();
        if name.trim().is_empty() {
            return;
        }

        set_busy.set(true);
        set_error.set(None);
        spawn_local(async move {
            match actions::create_list(name.trim()).await {
                Ok(list) => alive.if_alive(|| {
                    set_title.set(String::new());
                    set_selected.set(Some(list.id));
                }),
                Err(message) => alive.if_alive(|| set_error.set(Some(message))),
            }
            alive.if_alive(|| set_busy.set(false));
            reload();
        });
    };

    let on_title = Callback::new(move |text: String| set_title.set(text));
    let on_create = Callback::new(move |()| create(()));

    // The row's two edits are the screen's, so the picker asks for a whole row rather than
    // drawing one it cannot make editable.
    let row = Callback::new(move |list: List| {
        view! {
            <ListRow
                list=list
                selected=selected.into()
                busy=busy.into()
                set_selected=set_selected
                set_error=set_error
                reload=Callback::new(move |()| {
                    set_busy.set(true);
                    reload();
                    set_busy.set(false);
                })
            />
        }
        .into_any()
    });

    let members: ChildrenFn =
        Arc::new(move || view! { <Members list_id=selected.into()/> }.into_any());
    let timeline: ChildrenFn =
        Arc::new(move || view! { <ListTimeline list_id=selected.into()/> }.into_any());

    view! {
        <ListsPage
            lists=lists.into()
            selected=selected.into()
            title=title.into()
            busy=busy.into()
            error=error.into()
            on_title=on_title
            on_create=on_create
            row=row
            members=members
            timeline=timeline
        />
    }
}
