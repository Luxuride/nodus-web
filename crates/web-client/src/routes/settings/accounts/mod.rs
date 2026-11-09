//! The accounts card: which identities this browser holds, and which one is on screen.
//!
//! Switching is a pointer move in the SDK registry, not a sign-in: the inactive account
//! keeps its tokens, its SDK and its timelines, so returning to it costs nothing. The
//! card therefore has no "loading" state of its own — the only thing that can fail is a
//! switch to an account whose SDK was never built in this page load, which is a message,
//! not a spinner.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::state::bridge;

#[component]
pub fn AccountsCard() -> impl IntoView {
    let (tick, set_tick) = signal(0u32);
    let (busy, set_busy) = signal(false);
    let (error, set_error) = signal(None::<String>);

    let list = move || {
        tick.get();
        bridge::account_list()
    };
    let active = move || {
        tick.get();
        bridge::active_key()
    };

    let choose = move |key: String| {
        set_busy.set(true);
        set_error.set(None);
        match bridge::switch(&key) {
            Ok(()) => set_tick.update(|generation| *generation += 1),
            Err(message) => set_error.set(Some(message)),
        }
        set_busy.set(false);
    };

    let sign_out = move |_: web_sys::MouseEvent| {
        set_busy.set(true);
        spawn_local(async move {
            bridge::session::sign_out();
            set_tick.update(|generation| *generation += 1);
            set_busy.set(false);
        });
    };

    view! {
        <nodus_view::components::panel::Panel title="Accounts">
                <ul class="flex flex-col gap-2">
                    {move || {
                        let current = active();
                        list()
                            .into_iter()
                            .map(|account| {
                                let key = account.key();
                                let is_current = current.as_deref() == Some(key.as_str());
                                let target = key.clone();
                                let label = format!("@{}", account.acct);
                                let host = account.host.clone();
                                let switch = move |_: web_sys::MouseEvent| choose(target.clone());
                                let badge = if is_current {
                                    view! { <span class="badge badge-primary badge-sm">"on screen"</span> }
                                        .into_any()
                                } else {
                                    view! {
                                        <button
                                            class="btn btn-ghost btn-xs"
                                            disabled=move || busy.get()
                                            on:click=switch
                                        >
                                            "Switch"
                                        </button>
                                    }
                                        .into_any()
                                };
                                view! {
                                    <li class="flex items-center justify-between gap-3 rounded-box bg-base-200 px-3 py-2">
                                        <div class="flex min-w-0 flex-col">
                                            <span class="truncate text-sm font-medium">{label}</span>
                                            <span class="truncate text-xs opacity-60">{host}</span>
                                        </div>
                                        {badge}
                                    </li>
                                }
                            })
                            .collect_view()
                    }}
                </ul>
                <p class="text-xs opacity-60">
                    "Signing in with another account adds it here. Switching keeps the \
                     timelines the other account already loaded."
                </p>
                <Show when=move || error.get().is_some()>
                    <p class="text-sm text-error">{move || error.get().unwrap_or_default()}</p>
                </Show>
                <button
                    class="btn btn-ghost btn-sm self-start"
                    disabled=move || busy.get()
                    on:click=sign_out
                >
                    "Sign out of this account"
                </button>
        </nodus_view::components::panel::Panel>
    }
}
