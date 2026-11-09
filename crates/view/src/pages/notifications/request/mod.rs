//! Accepting or refusing a follow request.
//!
//! The buttons belong to the notification the request arrived as, and to the strip that
//! repeats the queue above the list, so they live here rather than in either caller. What the
//! press *does* is the caller's: sending the answer, re-reading the list and reporting a
//! failure all need the instance, which this crate does not have.
//!
//! `on_answer` is the whole difference between the two paints. The client has it, so the
//! buttons send the account id and the decision; the server does not, so the same two buttons
//! keep their boxes and are disabled. A server paint therefore shows a reader that a decision
//! is being asked for without pretending it can be made there.

use leptos::prelude::*;

/// Two buttons and the answer they send, for one account.
///
/// `busy` covers every button that answers this account: the row's and the strip's, when the
/// caller shares one flag, so a press in one place cannot be repeated from the other.
#[component]
pub fn FollowRequestActions(
    /// The account asking to follow.
    account_id: String,
    /// Whether an answer is already on its way.
    #[prop(optional)]
    busy: Option<Signal<bool>>,
    /// Sends `(account id, accepted)`. Absent on the server.
    #[prop(optional)]
    on_answer: Option<Callback<(String, bool)>>,
) -> impl IntoView {
    // One condition for both buttons, so they can never disagree about whether the decision
    // is still open. Without `on_answer` it is never open on this paint.
    let waiting = move || busy.is_some_and(|busy| busy.get()) || on_answer.is_none();

    // A callback is `Fn` and both buttons send the same answer, so the id is cloned per press.
    let answer = move |accept: bool| {
        if let Some(on_answer) = on_answer {
            on_answer.run((account_id.clone(), accept));
        }
    };

    let yes = answer.clone();
    view! {
        <div class="flex gap-2">
            <button
                class="btn btn-primary btn-xs"
                disabled=waiting
                on:click=move |_| yes(true)
            >
                "Accept"
            </button>
            <button
                class="btn btn-ghost btn-xs"
                disabled=waiting
                on:click=move |_| answer(false)
            >
                "Refuse"
            </button>
        </div>
    }
}
