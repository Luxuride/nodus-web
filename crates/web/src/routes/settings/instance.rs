//! The instance's code of conduct, as the server states it.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::state::alive::Alive;
use crate::state::bridge::actions;

/// Reads `/api/v1/instance/rules` and states them plainly.
#[component]
pub fn InstanceRulesCard() -> impl IntoView {
    let (rules, set_rules) = signal(Vec::new());
    let alive = Alive::new();

    Effect::new(move |_| {
        spawn_local(async move {
            // A server that publishes no rules answers with an empty list; a failure is
            // equally quiet, because this card is information, not an operation.
            if let Ok(list) = actions::rules().await {
                alive.if_alive(|| set_rules.set(list));
            }
        });
    });

    view! {
        <Show when=move || !rules.get().is_empty()>
            <section class="card bg-base-100">
                <div class="card-body gap-2">
                    <h2 class="card-title text-base">"This instance's rules"</h2>
                    <ol class="list-decimal pl-5 text-sm">
                        <For each=move || rules.get() key=|rule| rule.id.clone() let:rule>
                            <li class="py-0.5">
                                <span>{rule.text.clone()}</span>
                                {rule.hint.clone().map(|hint| view! {
                                    <span class="block text-xs opacity-60">{hint}</span>
                                })}
                            </li>
                        </For>
                    </ol>
                </div>
            </section>
        </Show>
    }
}
