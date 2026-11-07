//! Account standing, and the three decisions a moderator makes here.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::{AdminAccount, ModerationAction};

use crate::state::alive::Alive;
use crate::state::bridge::admin;

/// Account list with silence, suspend and the undo.
#[component]
pub fn AccountList() -> impl IntoView {
    let alive = Alive::new();
    let (accounts, set_accounts) = signal(Vec::<AdminAccount>::new());
    let (local_only, set_local_only) = signal(false);
    let (busy, set_busy) = signal(true);
    let (error, set_error) = signal(None::<String>);

    let reload = Callback::new(move |()| {
        spawn_local(async move {
            let outcome = admin::accounts(local_only.get_untracked()).await;
            alive.if_alive(|| {
                set_busy.set(true);
                match outcome {
                    Ok(list) => {
                        set_accounts.set(list);
                        set_error.set(None);
                    }
                    Err(message) => set_error.set(Some(message)),
                }
                set_busy.set(false);
            });
        });
    });

    Effect::new(move |_| {
        local_only.get();
        reload.run(());
    });

    view! {
        <label class="label mb-2 cursor-pointer justify-start gap-2">
            <input
                type="checkbox"
                class="toggle toggle-sm"
                prop:checked=local_only
                on:change=move |event| set_local_only.set(event_target_checked(&event))
            />
            <span class="label-text">"Only accounts on this instance"</span>
        </label>

        <Show when=move || error.get().is_some()>
            <p class="text-sm text-error">{move || error.get().unwrap_or_default()}</p>
        </Show>

        <ul class="flex flex-col gap-2">
            <For each=move || accounts.get() key=|account| account.id.clone() let:account>
                {
                    let id = account.id.clone();
                    let name = account
                        .account
                        .as_ref()
                        .map_or_else(|| account.username.clone(), |public| format!("@{}", public.acct));
                    let standing = account.standing();
                    // A `Callback` rather than a closure: three buttons need the same
                    // handler, and a closure that captures `id` can only be moved once.
                    let act = Callback::new(move |action: ModerationAction| {
                        let id = id.clone();
                        spawn_local(async move {
                            if admin::moderate(&id, action).await.is_ok() {
                                reload.run(());
                            }
                        });
                    });
                    view! {
                        <li class="flex items-center justify-between gap-2 rounded-box bg-base-100 px-3 py-2 shadow-sm">
                            <div class="min-w-0">
                                <p class="truncate text-sm font-medium">{name.clone()}</p>
                                <p class="text-xs opacity-60">{standing}</p>
                            </div>
                            <div class="flex shrink-0 gap-1">
                                <button class="btn btn-ghost btn-xs" on:click=move |_| act.run(ModerationAction::Silence)>
                                    "Silence"
                                </button>
                                <button class="btn btn-ghost btn-xs" on:click=move |_| act.run(ModerationAction::Suspend)>
                                    "Suspend"
                                </button>
                                <button class="btn btn-ghost btn-xs" on:click=move |_| act.run(ModerationAction::Enable)>
                                    "Enable"
                                </button>
                            </div>
                        </li>
                    }
                }
            </For>
        </ul>

        <Show when=move || accounts.get().is_empty() && !busy.get()>
            <p class="text-sm opacity-70">"No accounts to show."</p>
        </Show>
    }
}
