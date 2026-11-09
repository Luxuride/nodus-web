//! Account standing: the list a moderator filters, and the three decisions it carries.
//!
//! The filter's state and each decision's request belong to the caller; the row, the toggle
//! and the empty line are markup and live here. A decision that has no callback — a paint
//! with no token behind it — keeps its button and loses only the press.

use leptos::prelude::*;
use nodus_instance::model::{AdminAccount, ModerationAction};

/// Account list with silence, suspend and the undo.
#[component]
pub fn AccountsPage(
    /// The accounts, as the caller last read them.
    #[prop(into)]
    accounts: Signal<Vec<AdminAccount>>,
    /// Whether the list is limited to accounts on this instance.
    #[prop(into)]
    local_only: Signal<bool>,
    /// Whether a read is already on its way: an empty list while it is is not "nothing here".
    #[prop(into)]
    busy: Signal<bool>,
    /// The last read's refusal, when it had one.
    #[prop(into)]
    error: Signal<Option<String>>,
    /// Flips the local-only filter. Absent where there is nothing to filter.
    #[prop(optional)]
    on_local_only: Option<Callback<bool>>,
    /// Moderates one account: the id, and which decision.
    #[prop(optional)]
    on_moderate: Option<Callback<(String, ModerationAction)>>,
) -> impl IntoView {
    view! {
        <label class="label mb-2 cursor-pointer justify-start gap-2">
            <input
                type="checkbox"
                class="toggle toggle-sm"
                prop:checked=move || local_only.get()
                disabled=on_local_only.is_none()
                on:change=move |event| {
                    if let Some(on_local_only) = on_local_only {
                        on_local_only.run(event_target_checked(&event));
                    }
                }
            />
            <span class="label-text">"Only accounts on this instance"</span>
        </label>

        <Show when=move || error.get().is_some()>
            <p class="text-sm text-error">{move || error.get().unwrap_or_default()}</p>
        </Show>

        <ul class="flex flex-col gap-2">
            <For each=move || accounts.get() key=|account| account.id.clone() let:account>
                {
                    let target = StoredValue::new(account.id.clone());
                    let name = account
                        .account
                        .as_ref()
                        .map_or_else(|| account.username.clone(), |public| format!("@{}", public.acct));
                    let standing = account.standing();
                    view! {
                        <li class="flex items-center justify-between gap-2 rounded-box bg-base-100 px-3 py-2 shadow-sm">
                            <div class="min-w-0">
                                <p class="truncate text-sm font-medium">{name}</p>
                                <p class="text-xs opacity-60">{standing}</p>
                            </div>
                            <div class="flex shrink-0 gap-1">
                                {moderate("Silence", ModerationAction::Silence, target, on_moderate)}
                                {moderate("Suspend", ModerationAction::Suspend, target, on_moderate)}
                                {moderate("Enable", ModerationAction::Enable, target, on_moderate)}
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

/// One decision, as a button that keeps its box when there is nothing to send.
fn moderate(
    label: &'static str,
    action: ModerationAction,
    target: StoredValue<String>,
    on_moderate: Option<Callback<(String, ModerationAction)>>,
) -> AnyView {
    view! {
        <button
            class="btn btn-ghost btn-xs"
            disabled=on_moderate.is_none()
            on:click=move |_| {
                if let Some(on_moderate) = on_moderate {
                    on_moderate.run((target.get_value(), action));
                }
            }
        >
            {label}
        </button>
    }
    .into_any()
}
