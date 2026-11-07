//! Accepting or refusing a follow request.
//!
//! The buttons live on the notification because that is where the request arrives; the
//! row is replaced by the parent's reload rather than mutating a list in place, so the
//! screen cannot show a request that the instance already answered.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::state::alive::Alive;
use crate::state::bridge::actions;

/// Two buttons and the failure text for them.
#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "a Leptos component receives its props by value"
)]
pub fn FollowRequestActions(
    account_id: String,
    on_done: Callback<()>,
    set_error: WriteSignal<Option<String>>,
) -> impl IntoView {
    let (busy, set_busy) = signal(false);
    let alive = Alive::new();

    let answer = {
        let account_id = account_id.clone();
        move |accept: bool| {
            let account_id = account_id.clone();

            set_busy.set(true);
            spawn_local(async move {
                let result = if accept {
                    actions::authorize_follow_request(&account_id)
                        .await
                        .map(drop)
                } else {
                    actions::reject_follow_request(&account_id).await
                };
                match result {
                    Ok(()) => alive.if_alive(|| on_done.run(())),
                    Err(message) => alive.if_alive(|| set_error.set(Some(message))),
                }
                alive.if_alive(|| set_busy.set(false));
            });
        }
    };

    let yes = answer.clone();
    view! {
        <div class="flex gap-2">
            <button
                class="btn btn-primary btn-xs"
                disabled=move || busy.get()
                on:click=move |_| yes(true)
            >
                "Accept"
            </button>
            <button
                class="btn btn-ghost btn-xs"
                disabled=move || busy.get()
                on:click=move |_| answer(false)
            >
                "Refuse"
            </button>
        </div>
    }
}
