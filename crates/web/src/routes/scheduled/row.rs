//! One queued post.

use leptos::prelude::*;
use nodus_instance::model::ScheduledStatus;

use crate::routes::format::relative_time;

/// A queued post with its time and a cancel button.
#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "a Leptos component receives its props by value"
)]
pub fn QueuedRow(item: ScheduledStatus, on_cancel: Callback<String>) -> impl IntoView {
    let id = item.id.clone();
    let when = format!(
        "{} · {}",
        item.scheduled_at,
        relative_time(&item.scheduled_at)
    );

    view! {
        <li class="flex items-start gap-3 p-3">
            <div class="min-w-0 flex-1">
                <p class="truncate text-sm">{item.summary()}</p>
                <p class="text-xs opacity-60">{when}</p>
            </div>
            <button class="btn btn-ghost btn-xs" on:click=move |_| on_cancel.run(id.clone())>
                "Cancel"
            </button>
        </li>
    }
}
