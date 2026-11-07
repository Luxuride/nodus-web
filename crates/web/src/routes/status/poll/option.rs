//! One option of a poll: its share of the votes and whether it is picked.
//!
//! The row owns the percentage arithmetic so the parent can stay a list and a footer, and
//! so the "no votes yet" case is handled once.

use leptos::prelude::*;

/// One option line, with a bar showing its share.
#[component]
pub(super) fn PollOptionRow(
    index: usize,
    title: String,
    votes: u64,
    shown: ReadSignal<nodus_instance::model::Status>,
    closed: bool,
    busy: ReadSignal<bool>,
    picked: ReadSignal<Vec<usize>>,
    pick: Callback<usize>,
) -> impl IntoView {
    let total: u64 = shown
        .get_untracked()
        .poll
        .map_or(0, |poll| {
            poll.options
                .iter()
                .filter_map(|option| option.votes_count)
                .sum()
        });
    // `checked_div` rather than a guard: a poll with no votes at all has no share, and the
    // expression says so without a branch that a later edit could invert.
    let share = votes
        .checked_mul(100)
        .and_then(|scaled| scaled.checked_div(total))
        .unwrap_or(0);

    view! {
        <button
            class="mb-2 w-full rounded-box border border-base-300 px-3 py-2 text-left text-sm hover:border-primary"
            class:border-primary=move || picked.get().contains(&index)
            disabled=move || closed || busy.get()
            on:click=move |_| pick.run(index)
        >
            <span class="flex items-center justify-between gap-2">
                <span>{title}</span>
                <span class="text-xs opacity-60">{format!("{votes} · {share}%")}</span>
            </span>
            <span class="mt-1 block h-1 w-full rounded bg-base-300">
                <span class="block h-1 rounded bg-primary" style=format!("width: {share}%")></span>
            </span>
        </button>
    }
}
