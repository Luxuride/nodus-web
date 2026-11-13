//! The audit log: read once per revision and handed to the shared page.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_client_core::model::AuditEntry;
use nodus_pages::pages::admin::audit::AuditPage;

use crate::state::alive::Alive;
use crate::state::bridge::{admin, revision};

#[component]
pub fn Audit() -> impl IntoView {
    let revision = revision();
    let (entries, set_entries) = signal(Vec::<AuditEntry>::new());
    let (error, set_error) = signal(None::<String>);

    Effect::new(move |_| {
        revision.get();
        let alive = Alive::new();
        spawn_local(async move {
            match admin::audit().await {
                Ok(list) => alive.if_alive(|| set_entries.set(list)),
                Err(message) => alive.if_alive(|| set_error.set(Some(message))),
            }
        });
    });

    view! { <AuditPage entries=entries error=error/> }
}
