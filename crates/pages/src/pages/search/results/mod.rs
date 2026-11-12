//! The three result lists: accounts, hashtags, and posts.
//!
//! A separate file because each list is a full card layout, and because the page above only
//! owns the question: keeping the lists here is what stops the search screen from becoming a
//! file nobody wants to edit.
//!
//! Records are the only section with anything to press, so its cards arrive as a call for one
//! status — the caller's card knows the age, the addresses and the verbs. Without one the
//! shared read-only card is drawn, which is the same boxes with the verbs it cannot own.

use leptos::prelude::*;
use nodus_client_core::model::{Account, Status, Tag};

use crate::display_name;
use crate::status_card::{StatusCard, StatusCardProps};

/// Accounts, hashtags and posts that matched.
#[component]
pub fn Results(
    /// The accounts the search found.
    accounts: Signal<Vec<Account>>,
    /// The posts the search found.
    statuses: Signal<Vec<Status>>,
    /// The hashtags the search found.
    tags: Signal<Vec<Tag>>,
    /// Whether a search has answered yet, which is what separates "nothing matched" from
    /// "nothing asked".
    searched: Signal<bool>,
    /// Formats a timestamp as an age, for the card this section draws itself. Absent when the
    /// caller has no clock to lend, and the card then carries no age rather than a made up
    /// one.
    age_of: Option<Callback<String, String>>,
    /// The client's own status card, drawn in place of the shared read-only one.
    card: Option<Callback<Status, AnyView>>,
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
                    {
                        card.map_or_else(
                            || {
                                let age = age_of.map_or_else(String::new, |age_of| {
                                    age_of.run(status.created_at.clone())
                                });
                                read_only_card(status.clone(), age)
                            },
                            |card| card.run(status.clone()),
                        )
                    }
                </For>
            </ul>
        </Show>
    }
}

/// The shared card for one post, with the seams this section has no way to fill.
///
/// The props are built as a struct rather than through `view!` because every seam is
/// optional: a setter for one takes the bare value, and a section that only has the option to
/// forward it would have to invent one.
fn read_only_card(status: Status, age: String) -> AnyView {
    let author_path = format!("/accounts/{}", status.account.id);
    let thread_path = format!("/status/{}", status.id);

    StatusCard(StatusCardProps {
        status,
        age,
        author_path,
        thread_path,
        on_navigate: None,
        on_open: None,
        on_boost: None,
        on_favourite: None,
        on_bookmark: None,
        on_reply: None,
        before_verbs: None,
        after_verbs: None,
        on_open_media: None,
        on_vote: None,
        interactive: false,
        body: None,
        extra: None,
    })
    .into_any()
}
