//! A paged list of accounts: followers, following, mutes and blocks all look like this.
//!
//! One component for four screens because the only differences are which bridge function
//! runs and which verb the button sends, and both are small enough to pass in.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_sdk::Command;
use nodus_sdk::page::PageKind;

use crate::state::alive::Alive;
use crate::state::bridge::{actions, dispatch, pages, revision};

mod row;

use row::PersonRow;

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

    /// The page store key for this list.
    fn page(self, id: &str) -> PageKind {
        match self {
            Self::Followers => PageKind::Followers(id.to_owned()),
            Self::Following => PageKind::Following(id.to_owned()),
            Self::Mutes => PageKind::Mutes,
            Self::Blocks => PageKind::Blocks,
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

/// One paged list of accounts.
#[component]
pub fn PeopleList(
    #[prop(into)]
    /// Whose list to read. Ignored for mutes and blocks, which are always the viewer's.
    id: String,
    /// Which list this is.
    kind: Kind,
) -> impl IntoView {
    let revision = revision();
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);
    let alive = Alive::new();

    // `StoredValue` rather than a captured `String`: with only `Copy` captures the
    // closure below is `Copy` too, so the effect and the "load more" button can share it.
    let id = StoredValue::new(id);
    let held = move || {
        revision.get();
        pages::accounts(&kind.page(&id.get_value()))
    };
    let accounts = move || held().map(|(list, _)| list).unwrap_or_default();
    let cursor = move || held().and_then(|(_, cursor)| cursor);

    let load = move |after: Option<String>| {
        let id = id.get_value();

        spawn_local(async move {
            let answer = pages::ensure_after(kind.page(&id), after).await;
            match answer {
                Ok(_) => alive.if_alive(|| set_loading.set(false)),
                Err(message) => alive.if_alive(|| {
                    set_error.set(Some(message));
                    set_loading.set(false);
                }),
            }
        });
    };

    Effect::new(move |_| load(None));

    let more = move |_| load(cursor());

    let undo = Callback::new(move |account_id: String| {
        spawn_local(async move {
            let answer = match kind {
                Kind::Mutes => actions::mute(&account_id, false).await.map(drop),
                Kind::Blocks => actions::block(&account_id, false).await.map(drop),
                Kind::Followers | Kind::Following => Ok(()),
            };
            match answer {
                Ok(()) => alive.if_alive(|| {
                    // The command owns the page, so the row leaves it and every screen
                    // reading that page agrees.
                    dispatch(Command::PageAccountDropped {
                        kind: kind.page(&id.get_value()),
                        account: account_id,
                    });
                }),
                Err(message) => alive.if_alive(|| set_error.set(Some(message))),
            }
        });
    });

    view! {
        <div class="flex flex-col gap-2">
            <Show when=move || !loading.get() && accounts().is_empty()>
                <p class="text-sm opacity-60">{kind.empty()}</p>
            </Show>

            <ul class="flex flex-col gap-1">
                <For each=accounts key=|account| account.id.clone() let:account>
                    <PersonRow account=account label=kind.undo() on_undo=undo/>
                </For>
            </ul>

            <Show when=move || cursor().is_some()>
                <button
                    class="btn btn-ghost btn-sm w-full"
                    disabled=move || loading.get()
                    on:click=more
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
