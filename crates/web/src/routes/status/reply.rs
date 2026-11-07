//! The reply composer under a thread.
//!
//! Split out because the composer owns state the thread does not care about (the draft,
//! whether a request is in flight) and because it is the part of the thread most likely
//! to grow: media, content warnings and polls all attach here.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::{Status, Visibility};

use crate::state::bridge::actions;

/// A text box that publishes a public reply to `parent` and hands the answer back.
#[component]
pub(super) fn ReplyBox(
    parent: String,
    set_replies: WriteSignal<Vec<Status>>,
    set_error: WriteSignal<Option<String>>,
) -> impl IntoView {
    let (draft, set_draft) = signal(String::new());
    let (posting, set_posting) = signal(false);

    let publish = move |()| {
        let text = draft.get_untracked();
        if text.trim().is_empty() {
            return;
        }
        let parent = parent.clone();
        set_posting.set(true);
        set_error.set(None);
        spawn_local(async move {
            match actions::publish(&text, Visibility::Public, Some(&parent)).await {
                Ok(reply) => {
                    set_draft.set(String::new());
                    set_replies.update(|replies| replies.push(reply));
                }
                Err(message) => set_error.set(Some(message)),
            }
            set_posting.set(false);
        });
    };

    view! {
        <form
            class="card border border-base-300 bg-base-100 shadow-sm"
            on:submit=move |event| {
                event.prevent_default();
                publish(());
            }
        >
            <div class="card-body gap-2 p-4">
                <textarea
                    class="textarea textarea-bordered w-full"
                    rows="3"
                    placeholder="Write a reply…"
                    prop:value=move || draft.get()
                    on:input=move |event| set_draft.set(event_target_value(&event))
                ></textarea>
                <div class="flex items-center justify-between gap-2">
                    <span class="text-xs opacity-60">"Public reply"</span>
                    <button
                        class="btn btn-primary btn-sm"
                        type="submit"
                        disabled=move || posting.get() || draft.get().trim().is_empty()
                    >
                        "Reply"
                    </button>
                </div>
            </div>
        </form>
    }
}
