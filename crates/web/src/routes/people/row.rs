//! One account in a people list.
//!
//! Its own file because the list around it is about paging and this row is about links;
//! keeping them together pushed the parent past clippy's line limit.

use leptos::prelude::*;
use nodus_instance::model::Account;

use crate::router::{self, Route};

/// A single row: avatar, handle, and — on the mute and block lists — an undo button.
#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "a Leptos component receives its props by value"
)]
pub fn PersonRow(
    /// Whose row this is.
    account: Account,
    /// The undo verb, or `None` when this list has nothing to undo.
    label: Option<&'static str>,
    /// Runs the undo for this account's id.
    on_undo: Callback<String>,
) -> impl IntoView {
    let route = Route::Account(account.id.clone());
    let href = route.path();
    // `display_name` is a plain `String` on the wire, and an empty one means "no name
    // set", so the handle is the friendlier label in that case.
    let name = if account.display_name.is_empty() {
        account.acct.clone()
    } else {
        account.display_name.clone()
    };
    let acct = account.acct.clone();
    let avatar = account.avatar.clone();
    let undo_id = account.id.clone();

    view! {
        <li class="flex items-center gap-2 rounded-box border border-base-300 bg-base-100 p-2">
            <a class="avatar" href=href.clone()>
                <div class="w-8 rounded-full bg-base-300">
                    <img src=avatar alt="" referrerpolicy="no-referrer"/>
                </div>
            </a>
            <a
                class="min-w-0 flex-1 truncate text-sm hover:underline"
                href=href.clone()
                on:click=move |event| router::link(event, route.clone())
            >
                {name}
                <span class="opacity-60">" @"{acct}</span>
            </a>
            {label.map(|verb| view! {
                <button class="btn btn-ghost btn-xs" on:click=move |_| on_undo.run(undo_id.clone())>
                    {verb}
                </button>
            })}
        </li>
    }
}
