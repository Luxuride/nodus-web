//! The three result lists.
//!
//! A separate file because each list is a full card layout, and because the page above
//! only owns the query: keeping the lists here is what stops the search screen from
//! becoming a file nobody wants to edit.

use leptos::prelude::*;
use nodus_instance::model::{Account, Status, Tag};

use crate::routes::format::display_name;
use crate::routes::status::StatusCard;

/// Accounts, hashtags and posts that matched.
#[component]
pub(super) fn Results(
    accounts: ReadSignal<Vec<Account>>,
    statuses: ReadSignal<Vec<Status>>,
    tags: ReadSignal<Vec<Tag>>,
    searched: ReadSignal<bool>,
) -> impl IntoView {
    view! {
        <Show when=move || {
            searched.get()
                && accounts.get().is_empty()
                && statuses.get().is_empty()
                && tags.get().is_empty()
        }>
            <p class="text-sm opacity-60">"Nothing matched."</p>
        </Show>

        <Show when=move || !accounts.get().is_empty()>
            <h2 class="text-sm font-semibold opacity-70">"Accounts"</h2>
            <ul class="flex flex-col gap-2">
                <For each=move || accounts.get() key=|account| account.id.clone() let:account>
                    <li class="card border border-base-300 bg-base-100 shadow-sm">
                        <a
                            class="card-body flex-row items-center gap-3 p-3"
                            href=format!("/accounts/{}", account.id)
                        >
                            <div class="avatar">
                                <div class="w-10 rounded-full bg-base-300">
                                    <img
                                        src=account.avatar.clone()
                                        alt=""
                                        referrerpolicy="no-referrer"
                                    />
                                </div>
                            </div>
                            <div class="min-w-0">
                                <p class="truncate text-sm font-semibold">
                                    {display_name(&account.display_name, &account.acct)}
                                </p>
                                <p class="truncate text-xs opacity-60">
                                    {format!("@{}", account.acct)}
                                </p>
                            </div>
                        </a>
                    </li>
                </For>
            </ul>
        </Show>

        <Show when=move || !tags.get().is_empty()>
            <h2 class="text-sm font-semibold opacity-70">"Hashtags"</h2>
            <div class="flex flex-wrap gap-2">
                <For each=move || tags.get() key=|tag| tag.name.clone() let:tag>
                    <span class="badge badge-outline">{format!("#{}", tag.name)}</span>
                </For>
            </div>
        </Show>

        <Show when=move || !statuses.get().is_empty()>
            <h2 class="text-sm font-semibold opacity-70">"Posts"</h2>
            <ul class="flex flex-col gap-3">
                <For each=move || statuses.get() key=|status| status.id.clone() let:status>
                    <StatusCard status=status/>
                </For>
            </ul>
        </Show>
    }
}
