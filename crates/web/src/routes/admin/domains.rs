//! The one federation lever Mastodon exposes to clients: allowed domains.
//!
//! Relays used to live here too. They have no API — `/methods/admin/relays/` is itself a
//! 404 page on the docs site and `/api/v1/admin/relays` answers the instance's HTML 404 —
//! so a client cannot offer them, and linking to the instance's own web UI instead sends
//! the reader somewhere this app cannot reach.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::DomainAllow;

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

    let allow = move |event: web_sys::SubmitEvent| {
        event.prevent_default();
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
    };

    view! {
        <div class="card border border-base-300 bg-base-100 shadow-sm">
            <div class="card-body gap-2 p-4">
                <h2 class="card-title text-base">"Allowed domains"</h2>
                <p class="text-sm opacity-70">
                    "An allow lets one domain through even when this instance blocks federation generally — for a domain you trust without wanting to federate with everyone."
                </p>
                <For each=move || items.get() key=|entry| entry.id.clone() let:entry>
                    {
                        let id = entry.id.clone();
                        let withdraw = move |_| {
                            let id = id.clone();
                            spawn_local(async move {
                                match admin::delete_domain_allow(&id).await {
                                    Ok(()) => set_items.update(|list| list.retain(|item| item.id != id)),
                                    Err(message) => set_error.set(Some(message)),
                                }
                            });
                        };
                        view! {
                            <div class="flex items-center gap-2 rounded-box bg-base-200 p-2 text-sm">
                                <span class="truncate">{entry.domain.clone()}</span>
                                <button class="btn btn-ghost btn-xs ml-auto" on:click=withdraw>"Withdraw"</button>
                            </div>
                        }
                    }
                </For>
                <Show when=move || items.get().is_empty()>
                    <p class="text-sm opacity-60">"No domain is allowed through a block."</p>
                </Show>
                <Show when=move || error.get().is_some()>
                    <p class="text-sm text-error">{move || error.get().unwrap_or_default()}</p>
                </Show>
                <form class="flex flex-wrap items-center gap-2" on:submit=allow>
                    <input
                        class="input input-bordered input-sm flex-1"
                        placeholder="example.social"
                        prop:value=domain
                        on:input=move |event| set_domain.set(event_target_value(&event))
                    />
                    <button class="btn btn-ghost btn-sm" disabled=move || busy.get()>"Allow"</button>
                </form>
            </div>
        </div>
    }
}
