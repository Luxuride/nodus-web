//! The directory screen: accounts the instance suggests, most active first.
//!
//! The instance offers two scopes for the same list — accounts here, and accounts it knows
//! elsewhere — so the screen is one list under two tabs. The scope is the caller's state: the
//! page reports the press and reads back which tab is lit, because changing the scope changes
//! which page of the store is read, and the page may not read one.
//!
//! The rows are the shared account rows. The directory is the one suggestion screen the client
//! had its own card for, and the card said what the row already says — an avatar, a name, a
//! handle and the line the account wrote about itself.

use leptos::prelude::*;
use nodus_instance::model::Account;

use crate::chrome::more_button::{MoreButton, MoreButtonProps};
use crate::chrome::Skeleton;
use crate::lists::account_row::{AccountRow, AccountRowProps};
use crate::{PageHeader, Screen};

/// Local or federated accounts, in the instance's own order of activity.
#[component]
pub fn DirectoryPage(
    /// Whether the "this instance" scope is showing.
    local_only: Signal<bool>,
    /// The accounts the scope holds, in the instance's order.
    accounts: Signal<Vec<Account>>,
    /// Whether a read is already on its way.
    loading: Signal<bool>,
    /// The last read's refusal, when it had one.
    error: Signal<Option<String>>,
    /// The cursor of the page after this one, when the instance has more. The directory pages
    /// by offset, so this is a count rather than an id.
    cursor: Signal<Option<String>>,
    /// Switches scope: `true` for accounts here, `false` for accounts known elsewhere.
    #[prop(optional)]
    on_scope: Option<Callback<bool>>,
    /// Opens the profile inside the app.
    #[prop(optional)]
    on_open: Option<Callback<String>>,
    /// Asks for the page after the given cursor.
    #[prop(optional)]
    on_more: Option<Callback<String>>,
) -> impl IntoView {
    view! {
        <Screen>
            <PageHeader
                title="People"
                description="Accounts this instance suggests, most active first."
            />

            <div role="tablist" class="tabs tabs-box w-fit">
                <button
                    role="tab"
                    class="tab"
                    class:tab-active=move || local_only.get()
                    disabled=on_scope.is_none()
                    on:click=move |_| {
                        if let Some(on_scope) = on_scope {
                            on_scope.run(true);
                        }
                    }
                >
                    "This instance"
                </button>
                <button
                    role="tab"
                    class="tab"
                    class:tab-active=move || !local_only.get()
                    disabled=on_scope.is_none()
                    on:click=move |_| {
                        if let Some(on_scope) = on_scope {
                            on_scope.run(false);
                        }
                    }
                >
                    "Known elsewhere"
                </button>
            </div>

            {move || {
                error
                    .get()
                    .map(|message| {
                        view! {
                            <div class="alert alert-error py-2 text-sm" role="alert">
                                <span>{message}</span>
                            </div>
                        }
                    })
            }}

            // Only before the first answer: a scope the caller already holds renders at once,
            // and an empty scope is a fact, not a wait.
            <Show when=move || loading.get() && accounts.get().is_empty()>
                <Skeleton count=4/>
            </Show>

            <ul class="flex flex-col gap-1">
                <For each=move || accounts.get() key=|account| account.id.clone() let:account>
                    {
                        // The row is handed its whole set of props: view syntax can pass an
                        // affordance but not the absence of one, and this page only knows what it
                        // was given.
                        AccountRow(AccountRowProps {
                            note: account.note.clone(),
                            account,
                            on_open,
                            after: None,
                        })
                    }
                </For>
            </ul>

            <Show when=move || !loading.get() && accounts.get().is_empty()>
                <div class="card border border-base-300 bg-base-100">
                    <div class="card-body items-center p-6 text-center">
                        <h2 class="card-title text-base">"Nobody to suggest yet"</h2>
                        <p class="text-sm opacity-70">
                            "This instance has not published a directory of accounts."
                        </p>
                    </div>
                </div>
            </Show>

            {move || {
                let next = cursor.get();
                MoreButton(MoreButtonProps {
                    cursor: next,
                    loading: loading.get(),
                    on_more,
                })
            }}
        </Screen>
    }
}
