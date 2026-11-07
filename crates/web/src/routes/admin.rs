//! The moderation screen: the queue, account standing, blocked domains.
//!
//! Reached only from Settings, and only rendered when a probe call succeeds. A menu entry
//! that leads to a 403 is worse than a screen that explains it is not for you.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::state::alive::Alive;
use crate::state::bridge::admin;

use self::accounts::AccountList;
use self::announcements::Announcements;
use self::domains::DomainList;
use self::reports::ReportQueue;
use self::server::Server;

mod accounts;
mod announcements;
mod domains;
mod reports;
mod server;

/// Which list is on screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Reports,
    Accounts,
    Domains,
    Announcements,
    Server,
}

impl Tab {
    const ALL: [Self; 5] = [
        Self::Reports,
        Self::Accounts,
        Self::Domains,
        Self::Announcements,
        Self::Server,
    ];

    const fn label(self) -> &'static str {
        match self {
            Self::Reports => "Reports",
            Self::Accounts => "Accounts",
            Self::Domains => "Domains",
            Self::Announcements => "Announcements",
            Self::Server => "Server",
        }
    }
}

/// The moderation screen.
#[component]
pub fn Admin() -> impl IntoView {
    let alive = Alive::new();
    let (tab, set_tab) = signal(Tab::Reports);
    let (allowed, set_allowed) = signal(None::<bool>);

    Effect::new(move |_| {
        spawn_local(async move {
            let verdict = admin::is_moderator().await;
            alive.if_alive(|| set_allowed.set(Some(verdict)));
        });
    });

    view! {
        <section class="mx-auto w-full max-w-2xl px-4 py-6">
            <h1 class="mb-4 text-2xl font-semibold">"Moderation"</h1>

            <Show when=move || allowed.get().is_none()>
                <p class="text-sm opacity-70">"Checking what this account may do…"</p>
            </Show>

            <Show when=move || allowed.get() == Some(false)>
                <p class="text-sm opacity-70">
                    "This account is not a moderator on this instance."
                </p>
            </Show>

            <Show when=move || allowed.get() == Some(true)>
                <div role="tablist" class="tabs tabs-box mb-4">
                    <For each=|| Tab::ALL key=|tab| format!("{tab:?}") let:kind>
                        <button
                            role="tab"
                            class="tab"
                            class:tab-active=move || tab.get() == kind
                            on:click=move |_| set_tab.set(kind)
                        >
                            {kind.label()}
                        </button>
                    </For>
                </div>

                {move || match tab.get() {
                    Tab::Reports => view! { <ReportQueue/> }.into_any(),
                    Tab::Accounts => view! { <AccountList/> }.into_any(),
                    Tab::Domains => view! { <DomainList/> }.into_any(),
                    Tab::Announcements => view! { <Announcements/> }.into_any(),
                    Tab::Server => view! { <Server/> }.into_any(),
                }}
            </Show>
        </section>
    }
}
