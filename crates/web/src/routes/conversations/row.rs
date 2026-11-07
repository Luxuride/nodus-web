//! One conversation row.

use leptos::prelude::*;
use nodus_instance::model::Conversation;

use crate::routes::format::relative_time;

/// The last message in a conversation, with its unread and mute state.
#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "a Leptos component receives its props by value"
)]
pub fn ConversationRow(
    item: Conversation,
    is_muted: bool,
    on_open: Callback<String>,
    on_mute: Callback<(String, bool)>,
) -> impl IntoView {
    let id = item.id.clone();
    let open_id = item.id.clone();
    let mute_id = item.id.clone();
    let names = item
        .accounts
        .iter()
        .map(|account| account.acct.clone())
        .collect::<Vec<_>>()
        .join(", ");
    let unread = item.unread;
    let last = item
        .last_status
        .as_ref()
        .map(|status| (status.content.clone(), relative_time(&status.created_at)));

    view! {
        <li class="flex items-start gap-3 p-3">
            <a
                class="min-w-0 flex-1"
                href="#"
                on:click=move |event| {
                    event.prevent_default();
                    on_open.run(open_id.clone());
                }
            >
                <p class="flex items-center gap-2 text-sm font-medium">
                    <span class="truncate">{names}</span>
                    <Show when=move || unread>
                        <span class="badge badge-primary badge-xs">"new"</span>
                    </Show>
                    <Show when=move || is_muted>
                        <span class="badge badge-ghost badge-xs">"muted"</span>
                    </Show>
                </p>
                <p class="truncate text-xs opacity-70">
                    {last.as_ref().map(|(text, _)| text.clone()).unwrap_or_default()}
                </p>
                <p class="text-xs opacity-50">
                    {last.map(|(_, when)| when).unwrap_or_default()}
                </p>
            </a>
            <button
                class="btn btn-ghost btn-xs"
                on:click=move |_| on_mute.run((mute_id.clone(), !is_muted))
            >
                {if is_muted { "Unmute" } else { "Mute" }}
            </button>
            <span class="hidden">{id}</span>
        </li>
    }
}
