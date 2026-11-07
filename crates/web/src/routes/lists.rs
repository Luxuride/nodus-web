//! Lists: hand-picked sets of accounts with a timeline each.
//!
//! Three panels — the lists themselves, the members of the selected one, and its
//! timeline — because a list is only useful if you can see who is on it. The selection is
//! one signal shared by the panels, so selecting a list reloads both without either panel
//! knowing about the other.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::List;

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

    let reload = move || {
        spawn_local(async move {
            match actions::lists().await {
                Ok(found) => {
                    // Keep the selection if it survived, otherwise open the first list:
                    // a panel with nothing selected looks like the screen failed to load.
                    let keep = selected
                        .get_untracked()
                        .filter(|id| found.iter().any(|list| &list.id == id));
                    set_selected.set(keep.or_else(|| found.first().map(|list| list.id.clone())));
                    set_lists.set(found);
                }
                Err(message) => set_error.set(Some(message)),
            }
        });
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
                Ok(list) => {
                    set_title.set(String::new());
                    set_selected.set(Some(list.id));
                }
                Err(message) => set_error.set(Some(message)),
            }
            set_busy.set(false);
            reload();
        });
    };

    view! {
        <section class="flex flex-col gap-4">
            <form
                class="flex items-end gap-2"
                on:submit=move |event| {
                    event.prevent_default();
                    create(());
                }
            >
                <label class="form-control w-full">
                    <span class="label-text text-xs opacity-60">"New list"</span>
                    <input
                        class="input input-bordered input-sm w-full"
                        placeholder="A name nobody else sees"
                        prop:value=move || title.get()
                        on:input=move |event| set_title.set(event_target_value(&event))
                    />
                </label>
                <button
                    class="btn btn-primary btn-sm"
                    type="submit"
                    disabled=move || busy.get() || title.get().trim().is_empty()
                >
                    "Create"
                </button>
            </form>

            <Show when=move || !lists.get().is_empty()>
                <ul class="flex flex-wrap gap-2">
                    <For each=move || lists.get() key=|list| list.id.clone() let:list>
                        <ListRow
                            list=list
                            selected=selected
                            set_selected=set_selected
                            busy=busy
                            set_error=set_error
                            reload=Callback::new(move |()| {
                                set_busy.set(true);
                                reload();
                                set_busy.set(false);
                            })
                        />
                    </For>
                </ul>
            </Show>

            <Show when=move || { !busy.get() && lists.get().is_empty() && error.get().is_none() }>
                <p class="text-sm opacity-60">"No lists yet. A list keeps a timeline of its own."</p>
            </Show>

            <Members list_id=selected/>
            <ListTimeline list_id=selected/>

            <Show when=move || error.get().is_some()>
                <p class="text-error text-xs">{move || error.get().unwrap_or_default()}</p>
            </Show>
        </section>
    }
}
