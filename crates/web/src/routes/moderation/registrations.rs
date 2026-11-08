//! Who may join, and who is waiting to be let in.
//!
//! The queue the instance is holding, and the honest statement that invitations cannot be
//! minted from a client: Mastodon serves them only to its own web UI, so there is no form
//! here and no button that would send the reader elsewhere.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::AdminAccount;

use crate::state::alive::Alive;
use crate::state::bridge::admin;

/// The approval queue, plus the way to the invitation page.
#[component]
pub fn Registrations() -> impl IntoView {
    let queue = view! { <Queue/> };
    view! {
        <div class="flex flex-col gap-4">
            <div class="card border border-base-300 bg-base-100 shadow-sm">
                <div class="card-body gap-2 p-4">
                    <h2 class="card-title text-base">"Invitations"</h2>
                    <p class="text-sm opacity-70">
                        "Mastodon has no invitation API: a client cannot mint or revoke one,
                        only the instance's own administration page can. Sign-ups that arrive
                        without an invitation appear in the queue below."
                    </p>
                    <p class="text-xs opacity-60">
                        "Generating one is a task for the instance's own administration pages:
                         Nodus can approve and reject arrivals, not invite them."
                    </p>
                </div>
            </div>
            {queue}
        </div>
    }
}

/// Accounts the instance is holding for approval.
#[component]
fn Queue() -> impl IntoView {
    let alive = Alive::new();
    let (items, set_items) = signal(Vec::<AdminAccount>::new());
    let (busy, set_busy) = signal(true);
    let (error, set_error) = signal(None::<String>);

    let reload = Callback::new(move |()| {
        spawn_local(async move {
            let outcome = admin::pending_accounts().await;
            alive.if_alive(|| {
                set_busy.set(true);
                match outcome {
                    Ok(list) => {
                        set_error.set(None);
                        set_items.set(list);
                    }
                    Err(message) => set_error.set(Some(message)),
                }
            });
        });
    });

    Effect::new(move |_| reload.run(()));

    let decide = Callback::new(move |(id, allow): (String, bool)| {
        spawn_local(async move {
            let outcome = if allow {
                admin::approve_account(&id).await
            } else {
                admin::reject_account(&id).await
            };
            alive.if_alive(|| match outcome {
                Ok(_) => reload.run(()),
                Err(message) => set_error.set(Some(message)),
            });
        });
    });

    view! {
        <div class="card border border-base-300 bg-base-100 shadow-sm">
            <div class="card-body gap-3 p-4">
                <div class="flex items-baseline justify-between gap-2">
                    <h2 class="card-title text-base">"Waiting for approval"</h2>
                    <button
                        class="btn btn-ghost btn-xs"
                        disabled=move || busy.get()
                        on:click=move |_| reload.run(())
                    >
                        "Refresh"
                    </button>
                </div>

                {move || error.get().map(|message| view! {
                    <p class="text-sm text-error">{message}</p>
                })}

                <Show when=move || busy.get() && items.get().is_empty()>
                    <p class="text-sm opacity-70">"Asking the instance…"</p>
                </Show>

                <Show when=move || !busy.get() && items.get().is_empty()>
                    <p class="text-sm opacity-70">
                        "Nobody is waiting. If this instance does not require approval, this
                        queue stays empty by design."
                    </p>
                </Show>

                <ul class="flex flex-col gap-2">
                    <For each=move || items.get() key=|account| account.id.clone() let:account>
                        <ApprovalRow
                            account=account
                            decide=decide
                            busy=busy
                        />
                    </For>
                </ul>
            </div>
        </div>
    }
}

/// One waiting account, with the two verdicts a moderator can give.
#[component]
fn ApprovalRow(
    account: AdminAccount,
    decide: Callback<(String, bool)>,
    busy: ReadSignal<bool>,
) -> impl IntoView {
    let id = account.id.clone();
    let allow = id_for(id.clone(), true, decide);
    let refuse = id_for(id, false, decide);
    let handle = format!("@{}", account.username);
    let joined = account
        .created_at
        .get(..10)
        .unwrap_or(&account.created_at)
        .to_owned();

    view! {
        <li class="flex items-center justify-between gap-3 rounded-box bg-base-200 p-3">
            <div class="min-w-0">
                <p class="truncate text-sm font-medium">{handle}</p>
                <p class="truncate text-xs opacity-60">{format!("registered {joined}")}</p>
            </div>
            <div class="flex shrink-0 gap-1">
                <button
                    class="btn btn-primary btn-xs"
                    disabled=move || busy.get()
                    on:click=move |_| allow.run(())
                >
                    "Approve"
                </button>
                <button
                    class="btn btn-ghost btn-xs"
                    disabled=move || busy.get()
                    on:click=move |_| refuse.run(())
                >
                    "Refuse"
                </button>
            </div>
        </li>
    }
}

/// A `Copy` handler for one account and one verdict.
///
/// `Callback` is the only handler type that can be handed to several buttons, and the
/// account id cannot be captured by both closures without moving it twice.
fn id_for(id: String, allow: bool, decide: Callback<(String, bool)>) -> Callback<()> {
    Callback::new(move |()| decide.run((id.clone(), allow)))
}
