//! The account section of Settings: the session, the identities this browser holds, and
//! the links to the mute and block lists.
//!
//! Every value arrives as a signal and every change leaves as a callback, so the caller
//! keeps the SDK registry and this crate stays free of storage.

use leptos::prelude::*;

use crate::components::panel::Panel;

/// One identity the browser can switch to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountItem {
    pub key: String,
    pub label: String,
    pub host: String,
    pub current: bool,
}

/// Who this browser is signed in as, and the way out.
#[component]
pub fn SessionCard(
    #[prop(into)] host: Signal<String>,
    #[prop(into)] acct: Signal<String>,
    #[prop(optional_no_strip)] on_sign_out: Option<Callback<()>>,
) -> impl IntoView {
    view! {
        <Panel title="Session">
            <dl class="grid grid-cols-[8rem_minmax(0,1fr)] gap-x-3 gap-y-1 text-sm">
                <dt class="opacity-60">"Instance"</dt>
                <dd class="truncate">{move || host.get()}</dd>
                <dt class="opacity-60">"Account"</dt>
                <dd class="truncate">{move || acct.get()}</dd>
            </dl>
            <div class="card-actions mt-2 justify-end">
                <button
                    class="btn btn-ghost btn-sm"
                    disabled=on_sign_out.is_none()
                    on:click=move |_| {
                        if let Some(on_sign_out) = on_sign_out {
                            on_sign_out.run(());
                        }
                    }
                >
                    "Sign out"
                </button>
            </div>
            <p class="text-xs opacity-60">
                "Signing out forgets the token stored in this browser."
            </p>
        </Panel>
    }
}

/// Which identities this browser holds, and which one is on screen.
#[component]
pub fn AccountsCard(
    #[prop(into)] accounts: Signal<Vec<AccountItem>>,
    #[prop(into)] busy: Signal<bool>,
    #[prop(into)] error: Signal<Option<String>>,
    #[prop(optional_no_strip)] on_switch: Option<Callback<String>>,
    #[prop(optional_no_strip)] on_sign_out: Option<Callback<()>>,
) -> impl IntoView {
    view! {
        <Panel title="Accounts">
            <ul class="flex flex-col gap-2">
                {move || {
                    accounts
                        .get()
                        .into_iter()
                        .map(|account| {
                            let key = account.key;
                            let badge = if account.current {
                                view! {
                                    <span class="badge badge-primary badge-sm">"on screen"</span>
                                }
                                    .into_any()
                            } else {
                                let for_switch = key.clone();
                                view! {
                                    <button
                                        class="btn btn-ghost btn-xs"
                                        disabled=move || busy.get()
                                        on:click=move |_| {
                                            if let Some(on_switch) = on_switch {
                                                on_switch.run(for_switch.clone());
                                            }
                                        }
                                    >
                                        "Switch"
                                    </button>
                                }
                                    .into_any()
                            };
                            view! {
                                <li class="flex items-center justify-between gap-3 rounded-box bg-base-200 px-3 py-2">
                                    <div class="flex min-w-0 flex-col">
                                        <span class="truncate text-sm font-medium">
                                            {account.label}
                                        </span>
                                        <span class="truncate text-xs opacity-60">
                                            {account.host}
                                        </span>
                                    </div>
                                    {badge}
                                </li>
                            }
                        })
                        .collect_view()
                }}
            </ul>
            <p class="text-xs opacity-60">
                "Signing in with another account adds it here. Switching keeps the timelines the other account already loaded."
            </p>
            <Show when=move || error.get().is_some()>
                <p class="text-sm text-error">{move || error.get().unwrap_or_default()}</p>
            </Show>
            <button
                class="btn btn-ghost btn-sm self-start"
                disabled=move || busy.get() || on_sign_out.is_none()
                on:click=move |_| {
                    if let Some(on_sign_out) = on_sign_out {
                        on_sign_out.run(());
                    }
                }
            >
                "Sign out of this account"
            </button>
        </Panel>
    }
}

/// The shortcuts to the mute and block lists.
#[component]
pub fn ModerationLinks(
    #[prop(optional_no_strip)] on_mutes: Option<Callback<()>>,
    #[prop(optional_no_strip)] on_blocks: Option<Callback<()>>,
) -> impl IntoView {
    view! {
        <Panel title="Moderation">
            <p class="text-sm opacity-70">
                "Accounts you have muted or blocked stay hidden until you undo it here."
            </p>
            <div class="card-actions justify-start gap-2">
                <a
                    class="btn btn-outline btn-sm"
                    href="/mutes"
                    on:click=move |event| {
                        if let Some(on_mutes) = on_mutes {
                            event.prevent_default();
                            on_mutes.run(());
                        }
                    }
                >
                    "Muted accounts"
                </a>
                <a
                    class="btn btn-outline btn-sm"
                    href="/blocks"
                    on:click=move |event| {
                        if let Some(on_blocks) = on_blocks {
                            event.prevent_default();
                            on_blocks.run(());
                        }
                    }
                >
                    "Blocked accounts"
                </a>
            </div>
        </Panel>
    }
}
