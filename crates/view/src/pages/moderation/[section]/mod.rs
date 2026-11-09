//! A paged list of accounts: followers, following, mutes and blocks all look like this.
//!
//! One page for four screens because the only differences are which list is read and which
//! verb a row carries, and both are the caller's to pass in. The page draws the empty line,
//! the rows, the paging control and the error line; the row itself is the shared account row,
//! and the list-only verb (unmute, unblock) sits in that row's trailing slot. A follower list
//! has nothing to undo, so it passes no verb and the row draws no control.
//!
//! There is no column of its own here: these lists are read inside the profile of the account
//! they belong to, or inside the settings screen for the viewer's own mutes and blocks, and
//! the screen around them belongs to whoever placed them.

use std::sync::Arc;

use leptos::prelude::*;
use nodus_instance::model::Account;

use crate::chrome::more_button::{MoreButton, MoreButtonProps};
use crate::lists::account_row::{AccountRow, AccountRowProps};

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
    /// What the list says when it holds nothing.
    #[must_use]
    pub fn empty(self) -> &'static str {
        match self {
            Self::Followers => "No followers yet.",
            Self::Following => "Not following anyone yet.",
            Self::Mutes => "Nobody is muted.",
            Self::Blocks => "Nobody is blocked.",
        }
    }

    /// The undo verb a row carries, or `None` when this list has nothing to undo.
    #[must_use]
    pub fn undo(self) -> Option<&'static str> {
        match self {
            Self::Mutes => Some("Unmute"),
            Self::Blocks => Some("Unblock"),
            Self::Followers | Self::Following => None,
        }
    }
}

/// One paged list of accounts.
#[component]
pub fn PeoplePage(
    /// Which list this is: the empty line and the rows' verb both follow from it.
    kind: Kind,
    /// The accounts, in the instance's order.
    accounts: Signal<Vec<Account>>,
    /// Whether a read is already on its way.
    loading: Signal<bool>,
    /// The last read's refusal, when it had one.
    error: Signal<Option<String>>,
    /// The cursor of the page after this one, when the instance has more.
    cursor: Signal<Option<String>>,
    /// Opens the profile inside the app.
    #[prop(optional)]
    on_open: Option<Callback<String>>,
    /// Runs the list's undo verb for one account id.
    #[prop(optional)]
    on_undo: Option<Callback<String>>,
    /// Asks for the page after the given cursor.
    #[prop(optional)]
    on_more: Option<Callback<String>>,
) -> impl IntoView {
    let verb = kind.undo();

    view! {
        <div class="flex flex-col gap-2">
            <Show when=move || !loading.get() && accounts.get().is_empty()>
                <p class="text-sm opacity-60">{kind.empty()}</p>
            </Show>

            <ul class="flex flex-col gap-1">
                <For each=move || accounts.get() key=|account| account.id.clone() let:account>
                    {
                        // The list is read inside the profile it belongs to, so the bio is
                        // already on the page above it: the row shows identity only, which is
                        // what these four lists showed before they were shared.
                        let note = String::new();
                        let row_id = account.id.clone();
                        // The row is handed its whole set of props: view syntax can pass an
                        // affordance but not the absence of one, and this list only knows what
                        // it was given.
                        AccountRow(AccountRowProps {
                            account,
                            note,
                            on_open,
                            after: undo_slot(row_id, verb, on_undo),
                        })
                    }
                </For>
            </ul>

            {move || {
                let next = cursor.get();
                MoreButton(MoreButtonProps {
                    cursor: next,
                    loading: loading.get(),
                    on_more,
                })
            }}

            <Show when=move || error.get().is_some()>
                <p class="text-error text-xs">{move || error.get().unwrap_or_default()}</p>
            </Show>
        </div>
    }
}

/// One row's undo control, or nothing at all when the list has no verb for it.
///
/// A list that has a verb but no callback — the server's paint of the mute list — still draws
/// the button, disabled, so the two paints differ by an affordance and not by a box.
fn undo_slot(
    id: String,
    verb: Option<&'static str>,
    on_undo: Option<Callback<String>>,
) -> Option<ChildrenFn> {
    let verb = verb?;
    Some(Arc::new(move || {
        let id = id.clone();
        view! {
            <button
                class="btn btn-ghost btn-xs"
                disabled=on_undo.is_none()
                on:click=move |_| {
                    if let Some(on_undo) = on_undo {
                        on_undo.run(id.clone());
                    }
                }
            >
                {verb}
            </button>
        }
        .into_any()
    }) as ChildrenFn)
}
