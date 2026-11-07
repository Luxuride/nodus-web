//! Who is on the selected list.
//!
//! Adding needs a search, because a list member has to be an account the instance
//! already knows about; removing needs only the id we already hold. Both go through the
//! same membership endpoint, differing by verb.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::Account;

use self::add::AddPeople;
use crate::state::alive::Alive;
use crate::state::bridge::actions;

mod add;

/// The membership panel: current members, then a search to add more.
#[component]
pub fn Members(list_id: ReadSignal<Option<String>>) -> impl IntoView {
    let (members, set_members) = signal(Vec::<Account>::new());
    let (busy, set_busy) = signal(false);
    let (error, set_error) = signal(None::<String>);
    // Bumped by the add panel: membership changed elsewhere, so the roster is stale.
    let (refresh, set_refresh) = signal(0_u32);
    let alive = Alive::new();

    let remove = {
        move |account_id: String| {
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
        }
    };

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
                                        disabled=move || busy.get()
                                        on:click=move |_| remove(id.clone())
                                    >
                                        "×"
                                    </button>
                                </li>
                            }
                        }
                    </For>
                </ul>

                <AddPeople list_id=list_id set_error=set_error on_add=Callback::new(move |()| {
                    set_refresh.update(|count| *count += 1);
                })/>

                <Show when=move || error.get().is_some()>
                    <p class="text-error text-xs">{move || error.get().unwrap_or_default()}</p>
                </Show>
            </div>
        </Show>
    }
}
