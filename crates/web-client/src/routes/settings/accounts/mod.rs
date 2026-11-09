//! The accounts card: which identities this browser holds, and which one is on screen.
//!
//! Switching is a pointer move in the SDK registry, not a sign-in: the inactive account
//! keeps its tokens, its SDK and its timelines, so returning to it costs nothing. The
//! card therefore has no "loading" state of its own — the only thing that can fail is a
//! switch to an account whose SDK was never built in this page load, which is a message,
//! not a spinner.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_pages::pages::settings::account::{AccountItem, AccountsCard as SharedAccountsCard};

use crate::state::bridge;

#[component]
pub fn AccountsCard() -> impl IntoView {
    let (tick, set_tick) = signal(0u32);
    let (busy, set_busy) = signal(false);
    let (error, set_error) = signal(None::<String>);

    let accounts = Signal::derive(move || {
        tick.get();
        let current = bridge::active_key();
        bridge::account_list()
            .into_iter()
            .map(|account| {
                let key = account.key();
                AccountItem {
                    current: current.as_deref() == Some(key.as_str()),
                    key,
                    label: format!("@{}", account.acct),
                    host: account.host.clone(),
                }
            })
            .collect()
    });

    let choose = move |key: String| {
        set_busy.set(true);
        set_error.set(None);
        match bridge::switch(&key) {
            Ok(()) => set_tick.update(|generation| *generation += 1),
            Err(message) => set_error.set(Some(message)),
        }
        set_busy.set(false);
    };

    let sign_out = move |()| {
        set_busy.set(true);
        spawn_local(async move {
            bridge::session::sign_out();
            set_tick.update(|generation| *generation += 1);
            set_busy.set(false);
        });
    };

    view! {
        <SharedAccountsCard
            accounts=accounts
            busy=Signal::derive(move || busy.get())
            error=Signal::derive(move || error.get())
            on_switch=Some(Callback::new(choose))
            on_sign_out=Some(Callback::new(sign_out))
        />
    }
}
