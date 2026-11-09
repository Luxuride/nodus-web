//! Who may join, and who is waiting to be let in.
//!
//! The queue the instance is holding, and the honest statement that invitations cannot be
//! minted from a client: Mastodon serves them only to its own web UI, so there is no form
//! here and no button that would send the reader elsewhere.
//!
//! Both cards — the invitations statement and the queue — are markup in `nodus-view`; this
//! module owns the read, the two verdicts and the refresh.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::AdminAccount;
use nodus_view::pages::moderation::registrations::RegistrationsPage;

use crate::state::alive::Alive;
use crate::state::bridge::admin;

/// The approval queue, plus the way to the invitation page.
#[component]
pub fn Registrations() -> impl IntoView {
    let alive = Alive::new();
    let (items, set_items) = signal(Vec::<AdminAccount>::new());
    let (busy, set_busy) = signal(true);
    let (error, set_error) = signal(None::<String>);

    let reload = Callback::new(move |()| {
        spawn_local(async move {
            let outcome = admin::pending_accounts().await;
            alive.if_alive(|| {
                set_busy.set(true);
                match outcome {
                    Ok(list) => {
                        set_error.set(None);
                        set_items.set(list);
                    }
                    Err(message) => set_error.set(Some(message)),
                }
                set_busy.set(false);
            });
        });
    });

    Effect::new(move |_| reload.run(()));

    let decide = Callback::new(move |(id, allow): (String, bool)| {
        spawn_local(async move {
            let outcome = if allow {
                admin::approve_account(&id).await
            } else {
                admin::reject_account(&id).await
            };
            alive.if_alive(|| match outcome {
                Ok(_) => reload.run(()),
                Err(message) => set_error.set(Some(message)),
            });
        });
    });

    view! {
        <RegistrationsPage
            items=items
            busy=busy
            error=error
            on_refresh=reload
            on_decide=decide
        />
    }
}
