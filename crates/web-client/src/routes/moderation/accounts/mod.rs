//! Account standing, and the three decisions a moderator makes here.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_client_core::model::{AdminAccount, ModerationAction};
use nodus_pages::pages::moderation::accounts::AccountsPage;

use crate::state::alive::Alive;
use crate::state::bridge::admin;

/// Account list with silence, suspend and the undo.
#[component]
pub fn AccountList() -> impl IntoView {
    let alive = Alive::new();
    let (accounts, set_accounts) = signal(Vec::<AdminAccount>::new());
    let (local_only, set_local_only) = signal(false);
    let (busy, set_busy) = signal(true);
    let (error, set_error) = signal(None::<String>);

    let reload = Callback::new(move |()| {
        spawn_local(async move {
            let outcome = admin::accounts(local_only.get_untracked()).await;
            alive.if_alive(|| {
                set_busy.set(true);
                match outcome {
                    Ok(list) => {
                        set_accounts.set(list);
                        set_error.set(None);
                    }
                    Err(message) => set_error.set(Some(message)),
                }
                set_busy.set(false);
            });
        });
    });

    // The toggle is a re-read, not a filter: the instance decides which accounts a
    // local-only list holds.
    Effect::new(move |_| {
        local_only.get();
        reload.run(());
    });

    let moderate = Callback::new(move |(id, action): (String, ModerationAction)| {
        spawn_local(async move {
            if admin::moderate(&id, action).await.is_ok() {
                reload.run(());
            }
        });
    });

    view! {
        <AccountsPage
            accounts=accounts
            local_only=local_only
            busy=busy
            error=error
            on_local_only=Callback::new(move |only: bool| set_local_only.set(only))
            on_moderate=moderate
        />
    }
}
