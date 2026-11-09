//! The one federation lever Mastodon exposes to clients: allowed domains.
//!
//! Relays used to live here too. They have no API — `/methods/admin/relays/` is itself a
//! 404 page on the docs site and `/api/v1/admin/relays` answers the instance's HTML 404 —
//! so a client cannot offer them, and linking to the instance's own web UI instead sends
//! the reader somewhere this app cannot reach.
//!
//! The panel, the list and the form are drawn in `nodus-view`; what stays here is the read,
//! the write and the list the two keep in step.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::DomainAllow;
use nodus_pages::pages::admin::domains::DomainsPage;

use crate::state::alive::Alive;
use crate::state::bridge::{admin, revision};

/// Domains let through a general block.
#[component]
pub fn Domains() -> impl IntoView {
    let revision = revision();
    let (items, set_items) = signal(Vec::<DomainAllow>::new());
    let (domain, set_domain) = signal(String::new());
    let (busy, set_busy) = signal(false);
    let (error, set_error) = signal(None::<String>);

    Effect::new(move |_| {
        revision.get();
        let alive = Alive::new();
        spawn_local(async move {
            match admin::domain_allows().await {
                Ok(list) => alive.if_alive(|| set_items.set(list)),
                Err(message) => alive.if_alive(|| set_error.set(Some(message))),
            }
        });
    });

    let allow = Callback::new(move |()| {
        let name = domain.get_untracked().trim().to_owned();
        if name.is_empty() {
            return;
        }
        set_busy.set(true);
        set_error.set(None);
        spawn_local(async move {
            match admin::create_domain_allow(&name).await {
                Ok(entry) => {
                    set_domain.set(String::new());
                    set_items.update(|list| list.push(entry));
                }
                Err(message) => set_error.set(Some(message)),
            }
            set_busy.set(false);
        });
    });

    let withdraw = Callback::new(move |id: String| {
        spawn_local(async move {
            match admin::delete_domain_allow(&id).await {
                Ok(()) => set_items.update(|list| list.retain(|item| item.id != id)),
                Err(message) => set_error.set(Some(message)),
            }
        });
    });

    view! {
        <DomainsPage
            items=items
            domain=domain
            busy=busy
            error=error
            on_domain=Callback::new(move |value: String| set_domain.set(value))
            on_allow=allow
            on_withdraw=withdraw
        />
    }
}
