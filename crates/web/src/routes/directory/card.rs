//! One suggested account.

use leptos::prelude::*;
use nodus_instance::model::Account;

use crate::router::{self, Route};
use crate::routes::feed::posts::plain_text;
use crate::routes::format::display_name;

/// A card that leads to the profile.
#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "a Leptos component receives its props by value"
)]
pub fn PersonCard(account: Account) -> impl IntoView {
    let route = Route::Account(account.id.clone());
    let href = route.path();
    let name = display_name(&account.display_name, &account.acct);
    let handle = format!("@{}", account.acct);
    let avatar = account.avatar.clone();
    let note = plain_text(&account.note);

    view! {
        <li class="card bg-base-100">
            <a
                class="card-body gap-2 p-3"
                href=href
                on:click=move |event| router::link(event, route.clone())
            >
                <div class="flex items-center gap-3">
                    <img class="size-10 rounded-full" src=avatar alt="" />
                    <div class="min-w-0">
                        <p class="truncate text-sm font-medium">{name}</p>
                        <p class="truncate text-xs opacity-60">{handle}</p>
                    </div>
                </div>
                <p class="line-clamp-3 text-xs opacity-70">{note}</p>
            </a>
        </li>
    }
}
