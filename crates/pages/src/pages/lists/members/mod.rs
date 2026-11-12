//! The roster of the selected list, and the screen's way of adding to it.
//!
//! The panel exists only while a list is open, because a membership list belongs to a list:
//! with nothing selected there is nothing to show and no address to write to. Removing is a
//! verb of the row — it names one account on this list — so it is a callback that arrives
//! with the id; adding people is a search with a form of its own, so it arrives as a slot
//! from the caller that owns its draft and its candidates.

use leptos::prelude::*;
use nodus_client_core::model::Account;

/// The accounts on the selected list.
#[component]
pub fn Members(
    /// The selected list; the panel is closed while it is `None`.
    list_id: Signal<Option<String>>,
    /// The accounts on that list.
    members: Signal<Vec<Account>>,
    /// Whether a write is on its way.
    busy: Signal<bool>,
    /// The last refusal, shown under the roster.
    error: Signal<Option<String>>,
    /// Removes one account from the list.
    #[prop(optional)]
    on_remove: Option<Callback<String>>,
    /// The screen's own way of adding people, under the roster.
    #[prop(optional)]
    add: Option<ChildrenFn>,
) -> impl IntoView {
    view! {
        <Show when=move || list_id.get().is_some()>
            <div class="flex flex-col gap-2 rounded-box border border-base-300 bg-base-100 p-3">
                <h3 class="text-sm font-semibold">"Members"</h3>
                <ul class="flex flex-wrap gap-1">
                    <For each=move || members.get() key=|account| account.id.clone() let:account>
                        {
                            let id = account.id.clone();
                            view! {
                                <li class="badge badge-lg gap-1">
                                    <span>{format!("@{}", account.acct)}</span>
                                    <button
                                        class="btn btn-ghost btn-xs"
                                        title="Remove from this list"
                                        disabled=move || busy.get() || on_remove.is_none()
                                        on:click=move |_| {
                                            if let Some(on_remove) = on_remove {
                                                on_remove.run(id.clone());
                                            }
                                        }
                                    >
                                        "×"
                                    </button>
                                </li>
                            }
                        }
                    </For>
                </ul>
                {add.clone().map(|add| add())}
                <Show when=move || error.get().is_some()>
                    <p class="text-error text-xs">{move || error.get().unwrap_or_default()}</p>
                </Show>
            </div>
        </Show>
    }
}
