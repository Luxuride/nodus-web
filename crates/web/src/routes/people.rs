//! A paged list of accounts: followers, following, mutes and blocks all look like this.
//!
//! One component for four screens because the only differences are which bridge function
//! runs and which verb the button sends, and both are small enough to pass in.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::Account;

use crate::router::{self, Route};
use crate::state::bridge::actions;

/// How the list was produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Who follows an account.
    Followers,
    /// Who an account follows.
    Following,
    /// Accounts the viewer muted.
    Mutes,
    /// Accounts the viewer blocked.
    Blocks,
}

impl Kind {
    /// The heading and the empty text.
    fn empty(self) -> &'static str {
        match self {
            Self::Followers => "No followers yet.",
            Self::Following => "Not following anyone yet.",
            Self::Mutes => "Nobody is muted.",
            Self::Blocks => "Nobody is blocked.",
        }
    }

    /// Whether rows carry an undo button.
    fn undo(self) -> Option<&'static str> {
        match self {
            Self::Mutes => Some("Unmute"),
            Self::Blocks => Some("Unblock"),
            Self::Followers | Self::Following => None,
        }
    }
}

/// Reads one page of the requested list.
async fn page(
    kind: Kind,
    id: &str,
    after: Option<&str>,
) -> Result<(Vec<Account>, Option<String>), String> {
    match kind {
        Kind::Followers => actions::followers(id, after)
            .await
            .map(|page| (page.items, page.next_max_id)),
        Kind::Following => actions::following(id, after)
            .await
            .map(|page| (page.items, page.next_max_id)),
        Kind::Mutes => actions::mutes(after)
            .await
            .map(|page| (page.items, page.next_max_id)),
        Kind::Blocks => actions::blocks(after)
            .await
            .map(|page| (page.items, page.next_max_id)),
    }
}

/// One paged list of accounts.
#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "a Leptos component receives its props by value"
)]
pub fn PeopleList(
    #[prop(into)]
    /// Whose list to read. Ignored for mutes and blocks, which are always the viewer's.
    id: String,
    /// Which list this is.
    kind: Kind,
) -> impl IntoView {
    let (accounts, set_accounts) = signal(Vec::<Account>::new());
    let (cursor, set_cursor) = signal(None::<String>);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);

    let load = {
        let id = id.clone();
        move |after: Option<String>| {
            let id = id.clone();
            set_loading.set(true);
            spawn_local(async move {
                match page(kind, &id, after.as_deref()).await {
                    Ok((found, next)) => {
                        set_cursor.set(next);
                        set_accounts.update(|list| list.extend(found));
                    }
                    Err(message) => set_error.set(Some(message)),
                }
                set_loading.set(false);
            });
        }
    };

    {
        let load = load.clone();
        Effect::new(move |_| load(None));
    }

    let more = {
        let load = load.clone();
        move |_| load(cursor.get_untracked())
    };

    let undo = move |account_id: String| {
        spawn_local(async move {
            let answer = match kind {
                Kind::Mutes => actions::mute(&account_id, false).await.map(drop),
                Kind::Blocks => actions::block(&account_id, false).await.map(drop),
                Kind::Followers | Kind::Following => Ok(()),
            };
            match answer {
                Ok(()) => set_accounts.update(|list| list.retain(|item| item.id != account_id)),
                Err(message) => set_error.set(Some(message)),
            }
        });
    };

    view! {
        <div class="flex flex-col gap-2">
            <Show when=move || !loading.get() && accounts.get().is_empty()>
                <p class="text-sm opacity-60">{kind.empty()}</p>
            </Show>

            <ul class="flex flex-col gap-1">
                <For each=move || accounts.get() key=|account| account.id.clone() let:account>
                    {
                        let route = Route::Account(account.id.clone());
                        let href = route.path();
                        let id = account.id.clone();
                        view! {
                            <li class="flex items-center gap-2 rounded-box border border-base-300 bg-base-100 p-2">
                                <a class="avatar" href=href.clone()>
                                    <div class="w-8 rounded-full bg-base-300">
                                        <img src=account.avatar.clone() alt="" referrerpolicy="no-referrer"/>
                                    </div>
                                </a>
                                <a
                                    class="min-w-0 flex-1 truncate text-sm hover:underline"
                                    href=href.clone()
                                    on:click=move |event| router::link(event, route.clone())
                                >
                                    {crate::routes::format::display_name(
                                        &account.display_name,
                                        &account.acct,
                                    )}
                                </a>
                                {kind.undo().map(|label| view! {
                                    <button
                                        class="btn btn-ghost btn-xs"
                                        on:click=move |_| undo(id.clone())
                                    >
                                        {label}
                                    </button>
                                })}
                            </li>
                        }
                    }
                </For>
            </ul>

            <Show when=move || cursor.get().is_some()>
                <button
                    class="btn btn-ghost btn-sm w-full"
                    disabled=move || loading.get()
                    on:click=more.clone()
                >
                    "Load more"
                </button>
            </Show>

            <Show when=move || error.get().is_some()>
                <p class="text-error text-xs">{move || error.get().unwrap_or_default()}</p>
            </Show>
        </div>
    }
}
