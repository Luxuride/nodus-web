//! Who may join, and who is waiting to be let in.
//!
//! The queue the instance is holding, and the honest statement that invitations cannot be
//! minted from a client: Mastodon serves them only to its own web UI, so there is no form
//! here, no button that would send the reader elsewhere, and that sentence is part of the
//! markup rather than something the caller has to remember to add.

use leptos::prelude::*;
use nodus_client_core::model::AdminAccount;

/// The approval queue, plus the way to the invitation page.
#[component]
pub fn RegistrationsPage(
    /// The accounts waiting for approval, as the caller last read them.
    #[prop(into)]
    items: Signal<Vec<AdminAccount>>,
    /// Whether a read is already on its way.
    #[prop(into)]
    busy: Signal<bool>,
    /// The last read's refusal, when it had one.
    #[prop(into)]
    error: Signal<Option<String>>,
    /// Asks the instance again. Absent where there is nowhere to ask.
    #[prop(optional)]
    on_refresh: Option<Callback<()>>,
    /// Lets one account in, or refuses it: the id, and which verdict.
    #[prop(optional)]
    on_decide: Option<Callback<(String, bool)>>,
) -> impl IntoView {
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

            <div class="card border border-base-300 bg-base-100 shadow-sm">
                <div class="card-body gap-3 p-4">
                    <div class="flex items-baseline justify-between gap-2">
                        <h2 class="card-title text-base">"Waiting for approval"</h2>
                        <button
                            class="btn btn-ghost btn-xs"
                            disabled=move || busy.get() || on_refresh.is_none()
                            on:click=move |_| {
                                if let Some(on_refresh) = on_refresh {
                                    on_refresh.run(());
                                }
                            }
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
                            <ApprovalRow account=account busy=busy decide=on_decide/>
                        </For>
                    </ul>
                </div>
            </div>
        </div>
    }
}

/// One waiting account, with the two verdicts a moderator can give.
#[component]
fn ApprovalRow(
    account: AdminAccount,
    busy: Signal<bool>,
    #[prop(optional_no_strip)] decide: Option<Callback<(String, bool)>>,
) -> impl IntoView {
    let target = StoredValue::new(account.id.clone());
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
                    disabled=move || busy.get() || decide.is_none()
                    on:click=move |_| {
                        if let Some(decide) = decide {
                            decide.run((target.get_value(), true));
                        }
                    }
                >
                    "Approve"
                </button>
                <button
                    class="btn btn-ghost btn-xs"
                    disabled=move || busy.get() || decide.is_none()
                    on:click=move |_| {
                        if let Some(decide) = decide {
                            decide.run((target.get_value(), false));
                        }
                    }
                >
                    "Refuse"
                </button>
            </div>
        </li>
    }
}
