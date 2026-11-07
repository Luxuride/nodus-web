//! Accounts waiting for a decision, when the instance requires approval.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::AdminAccount;

use crate::state::alive::Alive;
use crate::state::bridge::{admin, revision};

/// The approval queue: approve, or refuse.
#[component]
pub(super) fn Approvals() -> impl IntoView {
    let revision = revision();
    let (items, set_items) = signal(Vec::<AdminAccount>::new());
    let (busy, set_busy) = signal(false);
    let (error, set_error) = signal(None::<String>);

    Effect::new(move |_| {
        revision.get();
        let alive = Alive::new();
        spawn_local(async move {
            match admin::pending_accounts().await {
                Ok(list) => alive.if_alive(|| set_items.set(list)),
                Err(message) => alive.if_alive(|| set_error.set(Some(message))),
            }
        });
    });

    let decide = Callback::new(move |(id, allow): (String, bool)| {
        set_busy.set(true);
        set_error.set(None);
        spawn_local(async move {
            let outcome = if allow {
                admin::approve_account(&id).await
            } else {
                admin::reject_account(&id).await
            };
            match outcome {
                Ok(_) => set_items.update(|list| list.retain(|item| item.id != id)),
                Err(message) => set_error.set(Some(message)),
            }
            set_busy.set(false);
        });
    });

    view! {
        <div class="card border border-base-300 bg-base-100 shadow-sm">
            <div class="card-body gap-2 p-4">
                <h2 class="card-title text-base">"Waiting for approval"</h2>
                <p class="text-sm opacity-70">
                    "Accounts that signed up while this instance requires a moderator's decision."
                </p>

                <Show when=move || error.get().is_some()>
                    <p class="text-sm text-error">{move || error.get().unwrap_or_default()}</p>
                </Show>

                <For each=move || items.get() key=|account| account.id.clone() let:account>
                    <ApprovalRow account=account decide=decide busy=busy/>
                </For>

                <Show when=move || items.get().is_empty() && error.get().is_none()>
                    <p class="text-sm opacity-60">
                        "Nobody is waiting. An instance that does not require approval always answers this way."
                    </p>
                </Show>
            </div>
        </div>
    }
}

/// One application, with the two answers it can get.
#[component]
fn ApprovalRow(
    account: AdminAccount,
    decide: Callback<(String, bool)>,
    busy: ReadSignal<bool>,
) -> impl IntoView {
    let id = account.id;
    let name = if account.domain.is_none() {
        format!("@{}", account.username)
    } else {
        format!("@{}@{}", account.username, account.domain.clone().unwrap_or_default())
    };
    let email = account.email.unwrap_or_default();
    let approve = id_for(&id, true, decide);
    let reject = id_for(&id, false, decide);

    view! {
        <div class="flex flex-wrap items-center gap-2 rounded-box bg-base-200 p-2 text-sm">
            <div class="min-w-0">
                <p class="truncate font-medium">{name}</p>
                <p class="truncate text-xs opacity-60">{email}</p>
            </div>
            <div class="ml-auto flex gap-1">
                <button
                    class="btn btn-primary btn-xs"
                    disabled=move || busy.get()
                    on:click=move |_| approve.run(())
                >
                    "Approve"
                </button>
                <button
                    class="btn btn-ghost btn-xs"
                    disabled=move || busy.get()
                    on:click=move |_| reject.run(())
                >
                    "Refuse"
                </button>
            </div>
        </div>
    }
}

/// Binds one answer to one account, so the two buttons cannot disagree about the id.
fn id_for(id: &str, allow: bool, decide: Callback<(String, bool)>) -> Callback<()> {
    let id = id.to_owned();
    Callback::new(move |()| decide.run((id.clone(), allow)))
}
