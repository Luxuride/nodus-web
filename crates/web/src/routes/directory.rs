//! People the instance suggests.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::Account;

mod card;

use self::card::PersonCard;
use crate::state::alive::Alive;
use crate::state::bridge::actions;

/// Local or federated accounts, in the instance's own order of activity.
#[component]
pub fn Directory() -> impl IntoView {
    let (people, set_people) = signal(Vec::<Account>::new());
    let (local_only, set_local_only) = signal(true);
    let (offset, set_offset) = signal(0_u32);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);
    let alive = Alive::new();

    let load = move |local: bool, from: u32| {
        let alive = alive;
        spawn_local(async move {
            match actions::directory(local, from).await {
                Ok(page) => alive.if_alive(|| {
                    set_people.update(|held| {
                        if from == 0 {
                            held.clear();
                        }
                        for account in page {
                            if !held.iter().any(|known| known.id == account.id) {
                                held.push(account);
                            }
                        }
                    });
                    set_loading.set(false);
                }),
                Err(message) => alive.if_alive(|| {
                    set_error.set(Some(message));
                    set_loading.set(false);
                }),
            }
        });
    };

    // Reloading from the top is the right answer for a scope change: the list is a
    // ranking, so appending to a different ranking would mix two orders.
    Effect::new(move |_| load(local_only.get(), 0));

    let choose = move |local: bool| {
        set_local_only.set(local);
        set_offset.set(0);
        set_people.set(Vec::new());
        set_loading.set(true);
        set_error.set(None);
    };

    view! {
        <section class="flex flex-col gap-3">
            <h1 class="text-lg font-semibold">"People"</h1>

            <div role="tablist" class="tabs tabs-box w-fit">
                <button
                    role="tab"
                    class="tab"
                    class:tab-active=move || local_only.get()
                    on:click=move |_| choose(true)
                >
                    "Local"
                </button>
                <button
                    role="tab"
                    class="tab"
                    class:tab-active=move || !local_only.get()
                    on:click=move |_| choose(false)
                >
                    "Federated"
                </button>
            </div>

            {move || error.get().map(|message| view! {
                <div class="alert alert-error py-2 text-sm"><span>{message}</span></div>
            })}

            <Show when=move || loading.get()>
                <p class="text-sm opacity-60">"Looking around…"</p>
            </Show>

            <ul class="grid gap-3 sm:grid-cols-2">
                <For each=move || people.get() key=|account| account.id.clone() let:account>
                    <PersonCard account=account/>
                </For>
            </ul>

            <Show when=move || !loading.get() && people.get().is_empty()>
                <p class="text-sm opacity-60">"Nobody to suggest here yet."</p>
            </Show>

            <Show when=move || !people.get().is_empty()>
                <button
                    class="btn btn-sm self-start"
                    on:click=move |_| {
                        let next = offset.get_untracked() + 40;
                        set_offset.set(next);
                        load(local_only.get_untracked(), next);
                    }
                >
                    "More people"
                </button>
            </Show>
        </section>
    }
}
