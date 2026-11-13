//! The administrative action log.

use leptos::prelude::*;
use nodus_client_core::model::AuditEntry;

use crate::components::panel::Panel;

/// Recent actions, newest first.
#[component]
pub fn AuditPage(
    /// The rows, as the caller last read them.
    #[prop(into)]
    entries: Signal<Vec<AuditEntry>>,
    /// The last read's refusal.
    #[prop(into)]
    error: Signal<Option<String>>,
) -> impl IntoView {
    view! {
        <Panel
            title="Audit log"
            description="Administrative and moderation actions recorded by this server."
        >
            <Show when=move || error.get().is_some()>
                <p class="text-sm text-error">{move || error.get().unwrap_or_default()}</p>
            </Show>
            <ul class="divide-y divide-base-300">
                <For each=move || entries.get() key=|entry| entry.id.clone() let:entry>
                    {
                        let when = entry.created_at.get(..19).unwrap_or(&entry.created_at).replace('T', " ");
                        let actor = entry.actor_id.clone().unwrap_or_else(|| "system".to_owned());
                        let target = entry
                            .target_id
                            .as_ref()
                            .map_or_else(|| entry.target_type.clone(), |id| format!("{} {id}", entry.target_type));
                        view! {
                            <li class="flex flex-col gap-1 py-2 text-sm">
                                <div class="flex flex-wrap items-baseline justify-between gap-2">
                                    <span class="font-medium">{entry.action.clone()}</span>
                                    <span class="text-xs opacity-60">{when}</span>
                                </div>
                                <span class="text-xs opacity-70">{format!("{target} · by {actor}")}</span>
                            </li>
                        }
                    }
                </For>
            </ul>
            <Show when=move || entries.get().is_empty()>
                <p class="text-sm opacity-60">"No administrative actions recorded yet."</p>
            </Show>
        </Panel>
    }
}
