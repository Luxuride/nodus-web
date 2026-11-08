//! Relays and allowed domains: the two levers that steer who this instance talks to.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::{DomainAllow, Relay};

use crate::state::alive::Alive;
use crate::state::bridge::{admin, revision};

/// Relays followed, plus the domains allowed through a block.
#[component]
pub(super) fn Federation() -> impl IntoView {
    view! {
        <div class="flex flex-col gap-3">
            <Relays/>
            <AllowedDomains/>
        </div>
    }
}

/// The relays this instance follows, and the form that adds one.
#[component]
fn Relays() -> impl IntoView {
    let revision = revision();
    let (items, set_items) = signal(Vec::<Relay>::new());
    let (url, set_url) = signal(String::new());
    let (busy, set_busy) = signal(false);
    let (error, set_error) = signal(None::<String>);

    Effect::new(move |_| {
        revision.get();
        let alive = Alive::new();
        spawn_local(async move {
            match admin::relays().await {
                Ok(list) => alive.if_alive(|| set_items.set(list)),
                Err(message) => alive.if_alive(|| set_error.set(Some(message))),
            }
        });
    });

    let follow = move |event: web_sys::SubmitEvent| {
        event.prevent_default();
        let inbox = url.get_untracked().trim().to_owned();
        if inbox.is_empty() {
            return;
        }
        set_busy.set(true);
        set_error.set(None);
        spawn_local(async move {
            match admin::add_relay(&inbox).await {
                Ok(relay) => {
                    set_url.set(String::new());
                    set_items.update(|list| list.push(relay));
                }
                Err(message) => set_error.set(Some(message)),
            }
            set_busy.set(false);
        });
    };

    view! {
        <div class="card border border-base-300 bg-base-100 shadow-sm">
            <div class="card-body gap-2 p-4">
                <h2 class="card-title text-base">"Relays"</h2>
                <p class="text-sm opacity-70">
                    "A relay forwards public posts between instances. Following one widens what this server sees, and what it stores."
                </p>
                <For each=move || items.get() key=|relay| relay.id.clone() let:relay>
                    {
                        let id = relay.id.clone();
                        let stop = move |_| {
                            let id = id.clone();
                            spawn_local(async move {
                                match admin::remove_relay(&id).await {
                                    Ok(()) => set_items.update(|list| list.retain(|item| item.id != id)),
                                    Err(message) => set_error.set(Some(message)),
                                }
                            });
                        };
                        view! {
                            <div class="flex items-center gap-2 rounded-box bg-base-200 p-2 text-sm">
                                <span class="badge badge-sm badge-ghost">{relay.state.clone()}</span>
                                <span class="truncate">{relay.inbox_url.clone()}</span>
                                <button class="btn btn-ghost btn-xs ml-auto" on:click=stop>"Stop"</button>
                            </div>
                        }
                    }
                </For>
                <Show when=move || items.get().is_empty()>
                    <p class="text-sm opacity-60">"No relays. A busy instance is fine without one."</p>
                </Show>
                <Show when=move || error.get().is_some()>
                    <p class="text-sm text-error">{move || error.get().unwrap_or_default()}</p>
                </Show>
                <form class="flex flex-wrap items-center gap-2" on:submit=follow>
                    <input
                        class="input input-bordered input-sm flex-1"
                        placeholder="https://relay.example/inbox"
                        prop:value=url
                        on:input=move |event| set_url.set(event_target_value(&event))
                    />
                    <button class="btn btn-ghost btn-sm" disabled=move || busy.get()>"Follow"</button>
                </form>
            </div>
        </div>
    }
}

/// Domains let through a general block.
#[component]
fn AllowedDomains() -> impl IntoView {
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
