//! The lists screen: the picker, its create form, and the panels of the selected list.
//!
//! A list is only useful if you can see who is on it and what they posted, so the screen is
//! one column over one selection: the lists themselves, the roster of the selected one, and
//! its timeline. The selection arrives as a signal because it is the only thing the three
//! panels share — picking another list is what makes both of the lower ones stale.
//!
//! The roster and the timeline arrive as slots rather than props. Each holds reads of its
//! own — the membership list, and a page of a timeline with the cursor that pages it — and
//! those belong to the caller, which knows the transport; the page keeps the column, the
//! heading, the create form and the picker, and passes on whatever the caller hangs under
//! the picker.

mod add;
mod members;
mod row;
mod timeline;

pub use add::AddPeople;
pub use members::Members;
pub use row::ListPickerRow;
pub use timeline::ListTimeline;

use leptos::prelude::*;
use nodus_client_core::model::List;

use crate::{PageHeader, Screen};

/// The lists screen.
#[component]
pub fn ListsPage(
    /// The lists this account has, in the order the instance returned them.
    lists: Signal<Vec<List>>,
    /// The list whose panels are open.
    selected: Signal<Option<String>>,
    /// What the reader has typed into the create form.
    title: Signal<String>,
    /// Whether a write is on its way, which disables every control that writes.
    busy: Signal<bool>,
    /// The last refusal, shown under the panels.
    error: Signal<Option<String>>,
    /// What the reader typed into the create form.
    #[prop(optional)]
    on_title: Option<Callback<String>>,
    /// Creates a list from the title. Absent where nothing can be written, and the form is
    /// then inert.
    #[prop(optional)]
    on_create: Option<Callback<()>>,
    /// Draws one picker row, for the caller that has edits to hang off it.
    #[prop(optional)]
    row: Option<Callback<List, AnyView>>,
    /// The roster of the selected list.
    #[prop(optional)]
    members: Option<ChildrenFn>,
    /// The selected list's timeline.
    #[prop(optional)]
    timeline: Option<ChildrenFn>,
) -> impl IntoView {
    // A row's border is a plain flag, because the shared row is redrawn rather than asked
    // again, so the list is keyed by the pair: a new selection rebuilds the two rows whose
    // flag flipped — and leaves the rest of the picker, and whatever is being typed into it,
    // alone. The flag is the caller's own read of the same signal, so the key and the row
    // agree on which list is open.
    let rows = move || {
        let here = selected.get();
        lists
            .get()
            .into_iter()
            .map(|list| {
                let open = here.as_deref() == Some(list.id.as_str());
                (list, open)
            })
            .collect::<Vec<(List, bool)>>()
    };

    view! {
        <Screen>
            <PageHeader
                title="Lists"
                description="Hand-picked accounts, each with a timeline of its own."
            />
            <NewListForm title=title busy=busy on_title=on_title on_create=on_create/>

            <Show when=move || !lists.get().is_empty()>
                <ul class="flex flex-wrap gap-2">
                    <For each=rows key=|row| (row.0.id.clone(), row.1) let:entry>
                        {
                            let (list, open) = entry;
                            match row {
                                Some(row) => row.run(list),
                                None => view! { <ListPickerRow list=list selected=open/> }.into_any(),
                            }
                        }
                    </For>
                </ul>
            </Show>

            <Show when=move || { !busy.get() && lists.get().is_empty() && error.get().is_none() }>
                <p class="text-sm opacity-60">"No lists yet. A list keeps a timeline of its own."</p>
            </Show>

            {members.map(|members| members())}
            {timeline.map(|timeline| timeline())}

            <Show when=move || error.get().is_some()>
                <p class="text-error text-xs">{move || error.get().unwrap_or_default()}</p>
            </Show>
        </Screen>
    }
}

/// The create form, split out so the screen reads as a layout rather than a form handler.
#[component]
fn NewListForm(
    title: Signal<String>,
    busy: Signal<bool>,
    on_title: Option<Callback<String>>,
    on_create: Option<Callback<()>>,
) -> impl IntoView {
    view! {
        <form
            class="flex items-end gap-2"
            on:submit=move |event| {
                stop_submit(&event);
                if let Some(on_create) = on_create {
                    on_create.run(());
                }
            }
        >
            <label class="form-control w-full">
                <span class="label-text text-xs opacity-60">"New list"</span>
                <input
                    class="input input-bordered input-sm w-full"
                    placeholder="A name nobody else sees"
                    prop:value=move || title.get()
                    disabled=on_title.is_none()
                    on:input=move |event| {
                        if let Some(on_title) = on_title {
                            on_title.run(event_target_value(&event));
                        }
                    }
                />
            </label>
            <button
                class="btn btn-primary btn-sm"
                type="submit"
                disabled=move || busy.get() || title.get().trim().is_empty() || on_create.is_none()
            >
                "Create"
            </button>
        </form>
    }
}

/// Keeps a submit from reloading the page, on the target that has a page to reload.
///
/// The server render never runs a listener, and its event type has no `prevent_default`; the
/// non-wasm build keeps a use of `event` so the argument is not reported as unused.
pub(super) fn stop_submit(event: &leptos::ev::SubmitEvent) {
    #[cfg(not(target_arch = "wasm32"))]
    let _ = event;
    #[cfg(target_arch = "wasm32")]
    event.prevent_default();
}
