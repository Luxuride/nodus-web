//! The dashboard: what this instance counted over the last week.
//!
//! The panel, the measure words and the dash fallback are drawn in `nodus-view`; what stays
//! here is asking the instance and remembering the answer.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_client_core::model::AdminMeasure;
use nodus_pages::pages::admin::overview::{MEASURES, OverviewPage};

use crate::state::alive::Alive;
use crate::state::bridge::{admin, revision};

/// How many days the dashboard counts over.
const DAYS: f64 = 7.0;

/// Dashboard counts over the last seven days.
#[component]
pub fn Overview() -> impl IntoView {
    let revision = revision();
    let (measures, set_measures) = signal(Vec::<AdminMeasure>::new());
    let (error, set_error) = signal(None::<String>);

    Effect::new(move |_| {
        revision.get();
        let alive = Alive::new();
        let keys: Vec<&str> = MEASURES.iter().map(|(key, _)| *key).collect();
        spawn_local(async move {
            match admin::measures(&keys, DAYS).await {
                Ok(list) => alive.if_alive(|| set_measures.set(list)),
                Err(message) => alive.if_alive(|| set_error.set(Some(message))),
            }
        });
    });

    view! {
        <OverviewPage measures=measures error=error/>
    }
}
