//! The report queue.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::{Report, ReportState};
use nodus_pages::pages::moderation::reports::ReportsPage;

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

    let resolve = Callback::new(move |id: String| {
        spawn_local(async move {
            // A failed resolve keeps the report on screen: the queue is the record of what
            // is still open.
            if admin::resolve_report(&id).await.is_ok() {
                reload.run(());
            }
        });
    });

    // The clock is the browser's; the page only asks how long ago a report was filed.
    let age_of = Callback::new(|created_at: String| relative_time(&created_at));

    view! {
        <ReportsPage
            reports=reports
            busy=busy
            error=error
            age_of=age_of
            on_resolve=resolve
        />
    }
}
