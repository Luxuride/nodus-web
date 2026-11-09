//! Posts waiting for their time.

use std::sync::Arc;

use leptos::prelude::*;
use nodus_state::page::{PageItems, PageKind};

use crate::pages::scheduled::ScheduledPage;
use crate::relative_age;

use super::super::{Action, Runtime};

#[component]
pub fn ScheduledScreen(runtime: Arc<dyn Runtime>) -> impl IntoView {
    let revision = runtime.revision();
    let loading = RwSignal::new(runtime.page(&PageKind::Scheduled).is_none());
    let error = RwSignal::new(None::<String>);
    let alive = crate::Alive::new();

    let items = {
        let runtime = Arc::clone(&runtime);
        move || {
            revision.get();
            runtime
                .page(&PageKind::Scheduled)
                .and_then(|page| match page.items {
                    PageItems::Scheduled(items) => Some(items),
                    _ => None,
                })
                .unwrap_or_default()
        }
    };
    let age_of = Callback::new({
        let runtime = Arc::clone(&runtime);
        move |created: String| relative_age(&created, runtime.now_millis())
    });

    let load: Callback<()> = Callback::new({
        let runtime = Arc::clone(&runtime);
        move |()| {
            loading.set(true);
            error.set(None);
            let done = Callback::new(move |result: Result<(), String>| {
                alive.if_alive(|| {
                    loading.set(false);
                    if let Err(message) = result {
                        error.set(Some(message));
                    }
                });
            });
            runtime.load_page(PageKind::Scheduled, None, done);
        }
    });

    Effect::new({
        let runtime = Arc::clone(&runtime);
        move |_| {
            if runtime.page(&PageKind::Scheduled).is_none() {
                load.run(());
            }
        }
    });

    let cancel = Callback::new({
        let runtime = Arc::clone(&runtime);
        move |id: String| {
            let done = Callback::new({
                let load = load;
                move |result: Result<crate::route::ActionResult, String>| match result {
                    Ok(_) => load.run(()),
                    Err(message) => error.set(Some(message)),
                }
            });
            runtime.action(Action::CancelScheduled(id), done);
        }
    });

    view! {
        <ScheduledPage
            items=Signal::derive(items)
            loading=loading.into()
            error=error.into()
            age_of=age_of
            on_cancel=cancel
        />
    }
}
