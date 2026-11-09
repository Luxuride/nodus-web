//! Lists: the picker, its roster and the selected list's timeline.

use std::sync::Arc;

use leptos::prelude::*;
use nodus_instance::model::{Account, List};

use crate::components::CardRenderer;
use crate::pages::lists::{AddPeople, ListPickerRow, ListTimeline, ListsPage, Members};

use super::super::{Action, ActionResult, Runtime, TimelineKind, TimelineReply};

/// The whole lists route.
#[component]
#[expect(
    clippy::too_many_lines,
    reason = "a route screen owns its layout and its reads"
)]
pub fn ListsScreen(runtime: Arc<dyn Runtime>, card: CardRenderer) -> impl IntoView {
    let runtime = StoredValue::new(runtime);
    let card = StoredValue::new(card);
    let painted = runtime.get_value().lists().unwrap_or_default();
    let (lists, set_lists) = signal(painted);
    let (selected, set_selected) = signal(
        lists
            .get_untracked()
            .first()
            .map(|list: &List| list.id.clone()),
    );
    let (title, set_title) = signal(String::new());
    let (busy, set_busy) = signal(false);
    let (error, set_error) = signal(None::<String>);
    let alive = crate::Alive::new();

    let reload = Callback::new(move |()| {
        set_busy.set(true);
        let done = Callback::new(move |result: Result<Vec<List>, String>| {
            alive.if_alive(|| {
                match result {
                    Ok(found) => {
                        let keep = selected
                            .get_untracked()
                            .filter(|id| found.iter().any(|list| &list.id == id));
                        set_selected
                            .set(keep.or_else(|| found.first().map(|list| list.id.clone())));
                        set_lists.set(found);
                    }
                    Err(message) => set_error.set(Some(message)),
                }
                set_busy.set(false);
            });
        });
        runtime.get_value().load_lists(done);
    });

    Effect::new(move |_| {
        if lists.get_untracked().is_empty() {
            reload.run(());
        }
    });

    let create = Callback::new(move |()| {
        let name = title.get_untracked().trim().to_owned();
        if name.is_empty() {
            return;
        }
        set_busy.set(true);
        set_error.set(None);
        let done = Callback::new(move |result: Result<ActionResult, String>| {
            alive.if_alive(|| {
                match result {
                    Ok(ActionResult::List(list)) => {
                        set_title.set(String::new());
                        set_selected.set(Some(list.id));
                    }
                    Ok(_) => {}
                    Err(message) => set_error.set(Some(message)),
                }
                set_busy.set(false);
            });
        });
        runtime.get_value().action(Action::CreateList(name), done);
    });

    let rename = Callback::new(move |(id, name): (String, String)| {
        set_busy.set(true);
        set_error.set(None);
        let done = Callback::new(move |result: Result<ActionResult, String>| {
            alive.if_alive(|| {
                match result {
                    Ok(_) => reload.run(()),
                    Err(message) => set_error.set(Some(message)),
                }
                set_busy.set(false);
            });
        });
        runtime
            .get_value()
            .action(Action::RenameList { id, title: name }, done);
    });

    let remove = Callback::new(move |id: String| {
        set_busy.set(true);
        set_error.set(None);
        let done = Callback::new(move |result: Result<ActionResult, String>| {
            alive.if_alive(|| {
                match result {
                    Ok(_) => reload.run(()),
                    Err(message) => set_error.set(Some(message)),
                }
                set_busy.set(false);
            });
        });
        runtime.get_value().action(Action::DeleteList(id), done);
    });

    let (members, set_members) = signal(Vec::<Account>::new());
    let (members_busy, set_members_busy) = signal(false);
    let (members_error, set_members_error) = signal(None::<String>);
    let selected_id = Signal::derive(move || selected.get());

    let load_members = Callback::new(move |()| {
        let Some(id) = selected.get_untracked() else {
            return;
        };
        set_members_busy.set(true);
        let done = Callback::new(move |result: Result<Vec<Account>, String>| {
            alive.if_alive(|| {
                match result {
                    Ok(found) => set_members.set(found),
                    Err(message) => set_members_error.set(Some(message)),
                }
                set_members_busy.set(false);
            });
        });
        runtime.get_value().load_list_accounts(id, done);
    });

    Effect::new(move |_| {
        if selected.get().is_some() {
            set_members.set(Vec::new());
            set_members_error.set(None);
            load_members.run(());
        }
    });

    let remove_member = Callback::new(move |account: String| {
        let Some(list) = selected.get_untracked() else {
            return;
        };
        set_members_busy.set(true);
        let done = Callback::new(move |result: Result<ActionResult, String>| {
            alive.if_alive(|| {
                match result {
                    Ok(_) => load_members.run(()),
                    Err(message) => set_members_error.set(Some(message)),
                }
                set_members_busy.set(false);
            });
        });
        runtime.get_value().action(
            Action::SetListMember {
                list,
                account,
                member: false,
            },
            done,
        );
    });

    let (query, set_query) = signal(String::new());
    let (candidates, set_candidates) = signal(Vec::<Account>::new());
    let search = Callback::new(move |()| {
        let text = query.get_untracked();
        if text.trim().is_empty() {
            return;
        }
        set_members_busy.set(true);
        let done = Callback::new(move |result: Result<Vec<Account>, String>| {
            alive.if_alive(|| {
                match result {
                    Ok(found) => set_candidates.set(found),
                    Err(message) => set_members_error.set(Some(message)),
                }
                set_members_busy.set(false);
            });
        });
        runtime.get_value().search_accounts(text, done);
    });

    let add_member = Callback::new(move |account: String| {
        let Some(list) = selected.get_untracked() else {
            return;
        };
        set_members_busy.set(true);
        let done = Callback::new(move |result: Result<ActionResult, String>| {
            alive.if_alive(|| {
                match result {
                    Ok(_) => load_members.run(()),
                    Err(message) => set_members_error.set(Some(message)),
                }
                set_members_busy.set(false);
            });
        });
        runtime.get_value().action(
            Action::SetListMember {
                list,
                account,
                member: true,
            },
            done,
        );
    });

    let (loading, set_loading) = signal(false);
    let (timeline_error, set_timeline_error) = signal(None::<String>);
    let snapshot = Signal::derive(move || {
        runtime.get_value().revision().get();
        selected
            .get()
            .map_or_else(TimelineReply::unavailable, |id| {
                runtime.get_value().timeline(&TimelineKind::List(id))
            })
    });
    let statuses = Signal::derive(move || snapshot.get().items);
    let cursor = Signal::derive(move || snapshot.get().cursor);

    let load_more = Callback::new(move |()| {
        let Some(id) = selected.get_untracked() else {
            return;
        };
        set_loading.set(true);
        let done = Callback::new(move |result: Result<(), String>| {
            alive.if_alive(|| {
                if let Err(message) = result {
                    set_timeline_error.set(Some(message));
                }
                set_loading.set(false);
            });
        });
        runtime
            .get_value()
            .load_timeline(TimelineKind::List(id), cursor.get_untracked(), done);
    });

    Effect::new(move |_| {
        if selected.get().is_some() && !snapshot.get().held && !snapshot.get().unavailable {
            load_more.run(());
        }
    });

    let row = Callback::new(move |list: List| {
        view! {
            <ListEdit
                list=list
                busy=Signal::derive(move || busy.get())
                on_rename=rename
                on_delete=remove
                on_select=Callback::new(move |id| set_selected.set(Some(id)))
            />
        }
        .into_any()
    });

    let members_slot: ChildrenFn = Arc::new(move || {
        view! {
            <Members
                list_id=selected_id
                members=members.into()
                busy=members_busy.into()
                error=members_error.into()
                on_remove=remove_member
                add=Arc::new(move || {
                    view! {
                        <AddPeople
                            query=query.into()
                            candidates=candidates.into()
                            busy=members_busy.into()
                            on_query=Callback::new(move |text| set_query.set(text))
                            on_search=search
                            on_add=add_member
                        />
                    }
                    .into_any()
                })
            />
        }
        .into_any()
    });

    let age_of = StoredValue::new(Callback::new(move |created_at: String| {
        crate::relative_age(&created_at, runtime.get_value().now_millis())
    }));

    let timeline: ChildrenFn = Arc::new(move || {
        view! {
            <ListTimeline
                list_id=selected_id
                statuses=statuses
                cursor=cursor
                loading=Signal::derive(move || loading.get())
                error=timeline_error.into()
                on_more=Callback::new(move |()| load_more.run(()))
                age_of=age_of.get_value()
                card=Some(Callback::new(move |status| card.get_value()(status)))
            />
        }
        .into_any()
    });

    view! {
        <ListsPage
            lists=lists.into()
            selected=selected_id
            title=title.into()
            busy=busy.into()
            error=error.into()
            on_title=Callback::new(move |text| set_title.set(text))
            on_create=create
            row=row
            members=members_slot
            timeline=timeline
        />
    }
}

#[component]
fn ListEdit(
    list: List,
    busy: Signal<bool>,
    on_rename: Callback<(String, String)>,
    on_delete: Callback<String>,
    on_select: Callback<String>,
) -> impl IntoView {
    let id = StoredValue::new(list.id.clone());
    let (editing, set_editing) = signal(false);
    let (draft, set_draft) = signal(list.title.clone());

    view! {
        <ListPickerRow
            list=list
            on_select=on_select
            controls=Arc::new({
                move || {
                    view! {
                        <Show when=move || editing.get()>
                            <input
                                class="input input-bordered input-xs w-32"
                                prop:value=move || draft.get()
                                on:input=move |event| set_draft.set(event_target_value(&event))
                                on:keydown=move |event| {
                                        if event.key() == "Enter" {
                                            on_rename.run((id.get_value(), draft.get_untracked()));
                                        set_editing.set(false);
                                    }
                                }
                            />
                        </Show>
                        <button
                            class="btn btn-ghost btn-xs"
                            title="Rename this list"
                            disabled=move || busy.get()
                            on:click=move |_| {
                                if editing.get_untracked() {
                                    on_rename.run((id.get_value(), draft.get_untracked()));
                                }
                                set_editing.update(|open| *open = !*open);
                            }
                        >
                            "✎"
                        </button>
                        <button
                            class="btn btn-ghost btn-xs"
                            title="Delete this list"
                            disabled=move || busy.get()
                            on:click=move |_| on_delete.run(id.get_value())
                        >
                            "×"
                        </button>
                    }
                    .into_any()
                }
            })
        />
    }
}
