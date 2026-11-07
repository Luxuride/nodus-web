//! One list in the picker: select it, rename it, or delete it.
//!
//! Rename is inline rather than a dialog: a list has exactly one editable field, so a
//! modal would be a screen to hold a text input.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::List;

use crate::state::alive::Alive;
use crate::state::bridge::actions;

/// A selectable list with its two edits.
#[component]
pub fn ListRow(
    list: List,
    selected: ReadSignal<Option<String>>,
    set_selected: WriteSignal<Option<String>>,
    busy: ReadSignal<bool>,
    set_error: WriteSignal<Option<String>>,
    reload: Callback<()>,
) -> impl IntoView {
    let id = list.id.clone();
    let title = list.title.clone();
    let (editing, set_editing) = signal(false);
    let (draft, set_draft) = signal(title.clone());
    let alive = Alive::new();

    let select = move |_| set_selected.set(Some(id.clone()));

    let rename = {
        let id = list.id.clone();
        Callback::new(move |()| {
            let name = draft.get_untracked();
            if name.trim().is_empty() || name == title {
                set_editing.set(false);
                return;
            }
            let id = id.clone();

            spawn_local(async move {
                match actions::rename_list(&id, name.trim()).await {
                    Ok(_) => alive.if_alive(|| set_editing.set(false)),
                    Err(message) => alive.if_alive(|| set_error.set(Some(message))),
                }
                alive.if_alive(|| reload.run(()));
            });
        })
    };

    let delete = {
        let id = list.id.clone();
        Callback::new(move |()| {
            let id = id.clone();

            spawn_local(async move {
                match actions::delete_list(&id).await {
                    Ok(()) => alive.if_alive(|| {
                        if selected.get_untracked().as_deref() == Some(id.as_str()) {
                            set_selected.set(None);
                        }
                    }),
                    Err(message) => alive.if_alive(|| set_error.set(Some(message))),
                }
                alive.if_alive(|| reload.run(()));
            });
        })
    };

    view! {
        <li
            class="flex items-center gap-1 rounded-box border border-base-300 bg-base-100 px-2 py-1"
            class:border-primary=move || selected.get().as_deref() == Some(list.id.as_str())
        >
            <Show
                when=move || editing.get()
                fallback=move || {
                    view! {
                        <button class="btn btn-ghost btn-xs" on:click=select.clone()>
                            {list.title.clone()}
                        </button>
                    }
                }
            >
                <input
                    class="input input-bordered input-xs w-40"
                    prop:value=move || draft.get()
                    on:input=move |event| set_draft.set(event_target_value(&event))
                />
                <button class="btn btn-ghost btn-xs" disabled=move || busy.get() on:click=move |_| rename.run(())>
                    "Save"
                </button>
            </Show>
            <button
                class="btn btn-ghost btn-xs"
                title="Rename this list"
                on:click=move |_| set_editing.update(|open| *open = !*open)
            >
                "✎"
            </button>
            <button
                class="btn btn-ghost btn-xs text-error"
                title="Delete this list"
                disabled=move || busy.get()
                on:click=move |_| delete.run(())
            >
                "×"
            </button>
        </li>
    }
}
