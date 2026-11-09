//! The dashboard: what this instance counted over the last week.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::AdminMeasure;

use crate::state::alive::Alive;
use crate::state::bridge::{admin, revision};

/// How many days the dashboard counts over.
const DAYS: f64 = 7.0;

/// The keys, with the words a moderator uses for them.
const KEYS: [(&str, &str); 5] = [
    ("new_users", "New accounts"),
    ("active_user_count", "Active accounts"),
    ("interactions", "Interactions"),
    ("opened_reports", "Reports opened"),
    ("resolved_reports", "Reports resolved"),
];

/// Dashboard counts over the last seven days.
#[component]
pub fn Overview() -> impl IntoView {
    let revision = revision();
    let (measures, set_measures) = signal(Vec::<AdminMeasure>::new());
    let (error, set_error) = signal(None::<String>);

    Effect::new(move |_| {
        revision.get();
        let alive = Alive::new();
        let keys: Vec<&str> = KEYS.iter().map(|(key, _)| *key).collect();
        spawn_local(async move {
            match admin::measures(&keys, DAYS).await {
                Ok(list) => alive.if_alive(|| set_measures.set(list)),
                Err(message) => alive.if_alive(|| set_error.set(Some(message))),
            }
        });
    });

    view! {
        <div class="flex flex-col gap-3">
            <Show when=move || error.get().is_some()>
                <p class="text-sm text-error">{move || error.get().unwrap_or_default()}</p>
            </Show>

            <div class="card border border-base-300 bg-base-100 shadow-sm">
                <div class="card-body gap-3 p-4">
                    <div class="flex items-baseline justify-between">
                        <h2 class="card-title text-base">"Last seven days"</h2>
                        <span class="text-xs opacity-50">"Counted by this instance"</span>
                    </div>
                    <dl class="grid grid-cols-2 gap-3 sm:grid-cols-3">
                        <For each=move || KEYS key=|(key, _)| *key let:entry>
                            {
                                let (key, label) = entry;
                                let reading = move || {
                                    measures
                                        .get()
                                        .into_iter()
                                        .find(|measure| measure.key == key)
                                };
                                view! {
                                    <div class="rounded-box bg-base-200 p-3">
                                        <dt class="text-xs opacity-60">{label}</dt>
                                        <dd class="text-xl font-semibold tabular-nums">
                                            {move || {
                                                reading()
                                                    .and_then(|measure| measure.total)
                                                    .unwrap_or_else(|| "—".to_owned())
                                            }}
                                        </dd>
                                        <dd class="text-xs opacity-50">
                                            {move || {
                                                reading()
                                                    .and_then(|measure| measure.previous_total)
                                                    .map_or_else(String::new, |previous| format!("was {previous}"))
                                            }}
                                        </dd>
                                    </div>
                                }
                            }
                        </For>
                    </dl>
                    <p class="text-xs opacity-50">
                        "A dash means this server does not report that measure, which is a fact about its version, not a failure."
                    </p>
                </div>
            </div>

        </div>
    }
}
