//! One list in the picker: select it, rename it, or delete it.
//!
//! The box itself is `nodus_view::pages::lists::ListPickerRow`: the name, the address it
//! selects through and the border that marks the open list. What stays here is the pair of
//! edits the shared row leaves open — the inline rename and the delete button — because the
//! draft and busy signals behind them are this screen's.

use std::sync::Arc;

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::List;
use nodus_view::pages::lists::ListPickerRow;

use crate::state::alive::Alive;
use crate::state::bridge::actions;

/// A selectable list with its two edits.
#[component]
pub fn ListRow(
    list: List,
    selected: Signal<Option<String>>,
    set_selected: WriteSignal<Option<String>>,
    busy: Signal<bool>,
    set_error: WriteSignal<Option<String>>,
    reload: Callback<()>,
) -> impl IntoView {
    let title = list.title.clone();
    // The picker rebuilds this row when the selection changes (it is keyed by the pair), so
    // the flag is read once per render rather than tracked.
    let open = selected.get_untracked().as_deref() == Some(list.id.as_str());
    let (editing, set_editing) = signal(false);
    let (draft, set_draft) = signal(title.clone());
    let alive = Alive::new();

    let on_select = Callback::new(move |id: String| set_selected.set(Some(id)));

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

    let controls: ChildrenFn = Arc::new(move || {
        view! {
            <Show when=move || editing.get()>
                <input
                    class="input input-bordered input-xs w-40"
                    prop:value=move || draft.get()
                    on:input=move |event| set_draft.set(event_target_value(&event))
                />
                <button
                    class="btn btn-ghost btn-xs"
                    disabled=move || busy.get()
                    on:click=move |_| rename.run(())
                >
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
        }
        .into_any()
    });

    view! {
        <ListPickerRow
            list=list
            selected=open
            on_select=on_select
            controls=controls
        />
    }
}
