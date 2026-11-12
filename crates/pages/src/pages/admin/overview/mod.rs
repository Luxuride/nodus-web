//! The dashboard: what this instance counted over the last week.
//!
//! The keys are the page's own data: the caller asks the instance for them and this page draws
//! them, so [`MEASURES`] is exported rather than copied into the client. A measure this
//! server does not report is drawn as a dash — a fact about the instance's version, not a
//! failure — and the caller's last refusal goes in the error line at the top.

use leptos::prelude::*;
use nodus_client_core::model::AdminMeasure;

/// The keys the dashboard asks for, with the words a moderator uses for them.
///
/// One list for both halves: the caller reads the keys from here and the page draws the
/// labels from here, so a key cannot be requested and then never shown.
pub const MEASURES: [(&str, &str); 5] = [
    ("new_users", "New accounts"),
    ("active_user_count", "Active accounts"),
    ("interactions", "Interactions"),
    ("opened_reports", "Reports opened"),
    ("resolved_reports", "Reports resolved"),
];

/// Dashboard counts over the last seven days.
#[component]
pub fn OverviewPage(
    /// The measures, as the caller last read them.
    #[prop(into)]
    measures: Signal<Vec<AdminMeasure>>,
    /// The last read's refusal, when it had one.
    #[prop(into)]
    error: Signal<Option<String>>,
) -> impl IntoView {
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
                        <For each=|| MEASURES key=|(key, _)| *key let:entry>
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
