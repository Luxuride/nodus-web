//! The report queue: what someone reported, and the one decision a moderator makes here.
//!
//! The cards are the same on both paints; the age in a card's header is the caller's clock,
//! so it arrives as `age_of` rather than being read here, and the "Resolve" press is the
//! caller's request, so it arrives as `on_resolve`. Where the press does not exist the button
//! keeps its box and is disabled.

use leptos::prelude::*;
use nodus_client_core::model::Report;

/// Open reports, each with the one button a moderator needs.
#[component]
pub fn ReportsPage(
    /// The reports, as the caller last read them.
    #[prop(into)]
    reports: Signal<Vec<Report>>,
    /// Whether a read is already on its way: an empty list while it is is not "nothing
    /// waiting".
    #[prop(into)]
    busy: Signal<bool>,
    /// The last read's refusal, when it had one.
    #[prop(into)]
    error: Signal<Option<String>>,
    /// Formats a timestamp as an age. Absent when the caller has no clock to lend.
    #[prop(optional)]
    age_of: Option<Callback<String, String>>,
    /// Marks one report resolved. Absent where there is nowhere to send it.
    #[prop(optional)]
    on_resolve: Option<Callback<String>>,
) -> impl IntoView {
    view! {
        <Show when=move || error.get().is_some()>
            <p class="text-sm text-error">{move || error.get().unwrap_or_default()}</p>
        </Show>

        <ul class="flex flex-col gap-3">
            <For each=move || reports.get() key=|report| report.id.clone() let:report>
                {
                    let target = StoredValue::new(report.id.clone());
                    let account = report
                        .target_account
                        .as_ref()
                        .map(|account| format!("@{}", account.acct))
                        .unwrap_or_default();
                    // The clock is the caller's, asked once per card: this page reads no time
                    // of its own.
                    let when = age_of.map_or_else(
                        String::new,
                        |age_of| age_of.run(report.created_at.clone()),
                    );
                    let category = report.category.clone();
                    let has_comment = !report.comment.is_empty();
                    let comment = report.comment.clone();
                    view! {
                        <li class="rounded-box bg-base-100 p-4 shadow-sm">
                            <div class="flex items-baseline justify-between gap-2">
                                <p class="text-sm font-medium">{account}</p>
                                <span class="text-xs opacity-60">{format!("{category} · {when}")}</span>
                            </div>
                            <Show when=move || has_comment>
                                <p class="mt-1 text-sm opacity-80">{comment.clone()}</p>
                            </Show>
                            <button
                                class="btn btn-sm mt-2"
                                disabled=on_resolve.is_none()
                                on:click=move |_| {
                                    // A failed resolve keeps the card on screen: the queue is
                                    // the record of what is still open.
                                    if let Some(on_resolve) = on_resolve {
                                        on_resolve.run(target.get_value());
                                    }
                                }
                            >
                                "Resolve"
                            </button>
                        </li>
                    }
                }
            </For>
        </ul>

        <Show when=move || reports.get().is_empty() && !busy.get()>
            <p class="text-sm opacity-70">"Nothing waiting."</p>
        </Show>
    }
}
