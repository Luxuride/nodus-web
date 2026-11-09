//! Domains the instance refuses.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::{DomainBlock, DomainSeverity};
use nodus_pages::pages::moderation::federation::FederationPage;

use crate::state::alive::Alive;
use crate::state::bridge::admin;

/// Blocked domains, with an add form.
#[component]
pub(super) fn BlockedList() -> impl IntoView {
    let alive = Alive::new();
    let (blocks, set_blocks) = signal(Vec::<DomainBlock>::new());
    let (domain, set_domain) = signal(String::new());
    let (suspend, set_suspend) = signal(true);
    let (busy, set_busy) = signal(true);
    let (error, set_error) = signal(None::<String>);

    let reload = Callback::new(move |()| {
        spawn_local(async move {
            let outcome = admin::domain_blocks().await;
            alive.if_alive(|| {
                set_busy.set(true);
                match outcome {
                    Ok(list) => {
                        set_blocks.set(list);
                        set_error.set(None);
                    }
                    Err(message) => set_error.set(Some(message)),
                }
                set_busy.set(false);
            });
        });
    });

    Effect::new(move |_| reload.run(()));

    let add = Callback::new(move |()| {
        let name = domain.get_untracked().trim().to_lowercase();
        if name.is_empty() {
            set_error.set(Some("Name the domain to block.".to_owned()));
            return;
        }
        let severity = if suspend.get_untracked() {
            DomainSeverity::Suspend
        } else {
            DomainSeverity::Silence
        };
        spawn_local(async move {
            match admin::create_domain_block(&name, severity).await {
                Ok(_) => {
                    alive.if_alive(|| set_domain.set(String::new()));
                    reload.run(());
                }
                Err(message) => alive.if_alive(|| set_error.set(Some(message))),
            }
        });
    });

    let remove = Callback::new(move |id: String| {
        spawn_local(async move {
            // A block that could not be removed is still in force; the list is the record
            // of that.
            if admin::delete_domain_block(&id).await.is_ok() {
                reload.run(());
            }
        });
    });

    view! {
        <FederationPage
            blocks=blocks
            domain=domain
            suspend=suspend
            busy=busy
            error=error
            on_domain=Callback::new(move |value: String| set_domain.set(value))
            on_suspend=Callback::new(move |entire: bool| set_suspend.set(entire))
            on_block=add
            on_remove=remove
        />
    }
}
