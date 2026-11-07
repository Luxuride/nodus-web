//! Invite links: create one, hand it out, revoke it.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::Invite;

use crate::state::alive::Alive;
use crate::state::bridge::{admin, revision};

mod row;

use row::InviteRow;

/// How long an invite lasts, in seconds, and what it says on the button.
const LIFETIMES: [(&str, Option<u64>); 4] = [
    ("30 days", Some(30 * 86_400)),
    ("7 days", Some(7 * 86_400)),
    ("1 day", Some(86_400)),
    ("never", None),
];

/// Invite links, and a form that makes one.
#[component]
pub(super) fn Invites() -> impl IntoView {
    let revision = revision();
    let (items, set_items) = signal(Vec::<Invite>::new());
    let (uses, set_uses) = signal(String::new());
    let (days, set_days) = signal(0_usize);
    let (busy, set_busy) = signal(false);
    let (error, set_error) = signal(None::<String>);
    // Precomputed: a turbofish inside `view!` reads as an attribute, not as a type argument.
    let choices = LIFETIMES
        .iter()
        .copied()
        .enumerate()
        .collect::<Vec<(usize, (&'static str, Option<u64>))>>();

    Effect::new(move |_| {
        revision.get();
        let alive = Alive::new();
        spawn_local(async move {
            match admin::invites().await {
                Ok(list) => alive.if_alive(|| set_items.set(list)),
                Err(message) => alive.if_alive(|| set_error.set(Some(message))),
            }
        });
    });

    let create = move |()| {
        // An empty box means "unlimited uses", which is not the same as zero.
        let max_uses = uses
            .get_untracked()
            .trim()
            .parse::<u32>()
            .ok()
            .filter(|count| *count > 0);
        let expires_in = LIFETIMES.get(days.get_untracked()).and_then(|(_, seconds)| *seconds);
        set_busy.set(true);
        set_error.set(None);
        spawn_local(async move {
            match admin::create_invite(max_uses, expires_in).await {
                Ok(invite) => {
                    set_uses.set(String::new());
                    set_items.update(|list| list.insert(0, invite));
                }
                Err(message) => set_error.set(Some(message)),
            }
            set_busy.set(false);
        });
    };

    view! {
        <div class="card border border-base-300 bg-base-100 shadow-sm">
            <div class="card-body gap-2 p-4">
                <h2 class="card-title text-base">"Invites"</h2>
                <p class="text-sm opacity-70">
                    "A link that lets someone join without waiting for the open sign-up form, which this instance may not have."
                </p>

                <Show when=move || error.get().is_some()>
                    <p class="text-sm text-error">{move || error.get().unwrap_or_default()}</p>
                </Show>

                <For each=move || items.get() key=|invite| invite.id.clone() let:invite>
                    <InviteRow invite=invite set_items=set_items set_error=set_error/>
                </For>

                <Show when=move || items.get().is_empty()>
                    <p class="text-sm opacity-60">"No invites yet."</p>
                </Show>

                <form
                    class="flex flex-wrap items-center gap-2"
                    on:submit=move |event| {
                        event.prevent_default();
                        create(());
                    }
                >
                    <input
                        class="input input-bordered input-sm w-32"
                        placeholder="uses (any)"
                        prop:value=uses
                        on:input=move |event| set_uses.set(event_target_value(&event))
                    />
                    <select
                        class="select select-bordered select-sm"
                        on:change=move |event| {
                            let picked = event_target_value(&event).parse::<usize>().unwrap_or(0);
                            set_days.set(picked);
                        }
                    >
                        <For each=move || choices.clone() key=|entry| entry.0 let:entry>
                            <option value=entry.0.to_string()>{entry.1 .0}</option>
                        </For>
                    </select>
                    <button class="btn btn-primary btn-sm" disabled=move || busy.get()>
                        "Create invite"
                    </button>
                </form>
            </div>
        </div>
    }
}

