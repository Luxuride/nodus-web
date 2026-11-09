//! The report queue.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::{Report, ReportState};

use crate::routes::format::relative_time;
use crate::state::alive::Alive;
use crate::state::bridge::admin;

/// Open reports, each with the one button a moderator needs.
#[component]
pub fn ReportQueue() -> impl IntoView {
    let alive = Alive::new();
    let (reports, set_reports) = signal(Vec::<Report>::new());
    let (busy, set_busy) = signal(true);
    let (error, set_error) = signal(None::<String>);

    let reload = Callback::new(move |()| {
        spawn_local(async move {
            let outcome = admin::reports(ReportState::Open).await;
            alive.if_alive(|| {
                set_busy.set(true);
                match outcome {
                    Ok(list) => {
                        set_reports.set(list);
                        set_error.set(None);
                    }
                    Err(message) => set_error.set(Some(message)),
                }
                set_busy.set(false);
            });
        });
    });

    Effect::new(move |_| reload.run(()));

    view! {
        <Show when=move || error.get().is_some()>
            <p class="text-sm text-error">{move || error.get().unwrap_or_default()}</p>
        </Show>

        <ul class="flex flex-col gap-3">
            <For each=move || reports.get() key=|report| report.id.clone() let:report>
                {
                    let id = report.id.clone();
                    let account = report
                        .target_account
                        .as_ref()
                        .map(|account| format!("@{}", account.acct))
                        .unwrap_or_default();
                    let when = relative_time(&report.created_at);
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
                                on:click=move |_| {
                                    let id = id.clone();
                                    spawn_local(async move {
                                        // A failed resolve keeps the report on screen:
                                        // the queue is the record of what is still open.
                                        if admin::resolve_report(&id).await.is_ok() {
                                            reload.run(());
                                        }
                                    });
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
