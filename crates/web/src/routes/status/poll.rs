//! A poll attached to a status: options, results, and one vote.
//!
//! The vote answers with the whole status, which is the only way to learn the new
//! counts: Mastodon has no "read this poll" endpoint, so the card replaces its copy with
//! the response instead of trusting a locally incremented number.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::Status;

use crate::state::alive::Alive;
use crate::state::bridge::actions;

mod option;

use option::PollOptionRow;

/// A poll, with results once the account has voted or the poll has closed.
#[component]
pub fn PollView(shown: ReadSignal<Status>, set_shown: WriteSignal<Status>) -> impl IntoView {
    let (picked, set_picked) = signal(Vec::<usize>::new());
    let (busy, set_busy) = signal(false);
    let (error, set_error) = signal(None::<String>);

    let choose = move |index: usize| {
        let multiple = shown.get_untracked().poll.is_some_and(|poll| poll.multiple);
        set_picked.update(|picked| {
            if !multiple {
                picked.clear();
                picked.push(index);
                return;
            }
            if let Some(position) = picked.iter().position(|choice| *choice == index) {
                picked.remove(position);
            } else {
                picked.push(index);
            }
        });
    };

    let vote = move |()| {
        let Some(poll) = shown.get_untracked().poll else {
            return;
        };
        let poll_id = poll.id.clone();
        let choices = picked.get_untracked();
        if choices.is_empty() {
            set_error.set(Some("Pick an option first.".to_owned()));
            return;
        }
        let alive = Alive::new();
        set_busy.set(true);
        set_error.set(None);
        spawn_local(async move {
            match actions::vote(&poll_id, &choices).await {
                Ok(status) => alive.if_alive(|| set_shown.set(status)),
                Err(message) => alive.if_alive(|| set_error.set(Some(message))),
            }
            alive.if_alive(|| set_busy.set(false));
        });
    };

    view! {
        <div class="rounded-box border border-base-300 p-3">
            <For
                each=move || {
                    shown
                        .get()
                        .poll
                        .map(|poll| poll.options.into_iter().enumerate().collect::<Vec<_>>())
                        .unwrap_or_default()
                }
                key=|(index, option)| (*index, option.title.clone())
                let:entry
            >
                {
                    let (index, option) = entry;
                    view! {
                        <PollOptionRow
                            index=index
                            title=option.title
                            votes=option.votes_count.unwrap_or(0)
                            shown=shown
                            closed=shown
                                .get_untracked()
                                .poll
                                .is_some_and(|poll| poll.voted || poll.expired)
                            busy=busy
                            picked=picked
                            pick=Callback::new(choose)
                        />
                    }
                }
            </For>
            <div class="flex items-center justify-between gap-2">
                <span class="text-xs opacity-60">
                    {move || {
                        let poll = shown.get().poll;
                        match poll {
                            Some(poll) if poll.expired => "Poll closed".to_owned(),
                            Some(poll) => format!("{} votes", poll.votes_count),
                            None => String::new(),
                        }
                    }}
                </span>
                <button
                    class="btn btn-primary btn-sm"
                    disabled=move || {
                        busy.get()
                            || shown.get().poll.is_none_or(|poll| poll.voted || poll.expired)
                    }
                    on:click=move |_| vote(())
                >
                    "Vote"
                </button>
            </div>
            <Show when=move || error.get().is_some()>
                <p class="text-error mt-1 text-xs">{move || error.get().unwrap_or_default()}</p>
            </Show>
        </div>
    }
}
