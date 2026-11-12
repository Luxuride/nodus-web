//! The one federation lever Mastodon exposes to clients: allowed domains.
//!
//! Relays used to live here too. They have no API — `/methods/admin/relays/` is itself a 404
//! page on the docs site and `/api/v1/admin/relays` answers the instance's HTML 404 — so a
//! client cannot offer them, and linking to the instance's own web UI instead sends the reader
//! somewhere this app cannot reach.
//!
//! The typed domain is the caller's state; the list, the withdrawal and the form are markup,
//! so the form's submit, its one input and its one press stay in one place.

use leptos::prelude::*;
use nodus_client_core::model::DomainAllow;

use crate::components::panel::Panel;

/// Domains let through a general block.
#[component]
pub fn DomainsPage(
    /// The allows this instance holds, as the caller last read them.
    #[prop(into)]
    items: Signal<Vec<DomainAllow>>,
    /// The domain being typed.
    #[prop(into)]
    domain: Signal<String>,
    /// Whether a write is already on its way.
    #[prop(into)]
    busy: Signal<bool>,
    /// The last read's or write's refusal, when it had one.
    #[prop(into)]
    error: Signal<Option<String>>,
    /// Records the typed domain. Absent where there is nowhere to keep it.
    #[prop(optional)]
    on_domain: Option<Callback<String>>,
    /// Allows the typed domain. Absent where nothing can be sent.
    #[prop(optional)]
    on_allow: Option<Callback<()>>,
    /// Withdraws one allow. Absent where nothing can be sent.
    #[prop(optional)]
    on_withdraw: Option<Callback<String>>,
) -> impl IntoView {
    view! {
        <Panel
            title="Allowed domains"
            description="An allow lets one domain through even when this instance blocks federation generally — for a domain you trust without wanting to federate with everyone."
        >
            <For each=move || items.get() key=|entry| entry.id.clone() let:entry>
                {
                    let target = StoredValue::new(entry.id.clone());
                    let name = entry.domain.clone();
                    view! {
                        <div class="flex items-center gap-2 rounded-box bg-base-200 p-2 text-sm">
                            <span class="truncate">{name}</span>
                            <button
                                class="btn btn-ghost btn-xs ml-auto"
                                disabled=on_withdraw.is_none()
                                on:click=move |_| {
                                    if let Some(on_withdraw) = on_withdraw {
                                        on_withdraw.run(target.get_value());
                                    }
                                }
                            >
                                "Withdraw"
                            </button>
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
            <form
                class="flex flex-wrap items-center gap-2"
                on:submit=move |event| {
                    stop_submit(&event);
                    if let Some(on_allow) = on_allow {
                        on_allow.run(());
                    }
                }
            >
                <input
                    class="input input-bordered input-sm flex-1"
                    placeholder="example.social"
                    prop:value=move || domain.get()
                    disabled=on_domain.is_none()
                    on:input=move |event| {
                        if let Some(on_domain) = on_domain {
                            on_domain.run(event_target_value(&event));
                        }
                    }
                />
                <button
                    class="btn btn-ghost btn-sm"
                    disabled=move || busy.get() || on_allow.is_none()
                >
                    "Allow"
                </button>
            </form>
        </Panel>
    }
}

/// Keeps a submit from reloading the page, on the target that has a page to reload.
///
/// The server render never runs a listener, and its event type has no `prevent_default`; the
/// non-wasm build keeps a use of `event` so the argument is not reported as unused.
fn stop_submit(event: &leptos::ev::SubmitEvent) {
    #[cfg(not(target_arch = "wasm32"))]
    let _ = event;
    #[cfg(target_arch = "wasm32")]
    event.prevent_default();
}
