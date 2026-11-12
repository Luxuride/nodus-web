//! One direct conversation, shared by the server paint and the client mount.
//!
//! A conversation has no thread page of its own, so the row is the whole message list in
//! miniature: who is in it, the newest status, and the two facts a reader needs before
//! opening it — whether anything in it is unread, and how old the newest status is.
//!
//! The newest status arrives as a slot because the two paints hold it in different shapes:
//! the server has a rendered status card, the client its own preview. The row does not
//! decide which, and a conversation with no status at all renders nothing there — the same
//! boxes with the slot empty — rather than a different row.
//!
//! The mute toggle is a slot for the same reason: the mute state is not part of the
//! conversation payload, so the screen that remembers it is the only one that can undo it.
//! The row draws the badge a muted conversation carries — that state is passed in, because a
//! reader has to see it — and leaves the verb to the trailing slot.

use leptos::prelude::*;
use nodus_client_core::model::Conversation;

/// Everyone in a conversation, as one line.
///
/// The server used to summarise this as "one and three more"; the client lists the handles,
/// and the client is what has to stay still across a load, so the list wins. A conversation
/// with no participants is still a conversation, so it keeps a neutral name.
#[must_use]
fn participants(conversation: &Conversation) -> String {
    if conversation.accounts.is_empty() {
        return "a conversation".to_owned();
    }
    conversation
        .accounts
        .iter()
        .map(|account| account.acct.clone())
        .collect::<Vec<_>>()
        .join(", ")
}

/// One conversation: the participants, the last message, and its read and mute state.
///
/// `age` is the last message's age in words, from [`crate::relative_age`]; this crate reads
/// no clock. `on_open` receives the conversation id, and `on_read` marks it read — without
/// it the mark-as-read button keeps its label and its box but is disabled, so the server's
/// row and the client's do not differ by a control's width. `muted` is the one piece of the
/// mute state the row can draw on its own, and `after` is where the screen that owns the
/// toggle puts it.
#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "a Leptos component receives its props by value"
)]
pub fn ConversationRow(
    /// The conversation this row shows.
    conversation: Conversation,
    /// The last message's age in words ("3h").
    age: String,
    /// Whether the viewer muted this conversation, which the row says with a badge.
    muted: bool,
    /// Opens the conversation inside the app.
    #[prop(optional)]
    on_open: Option<Callback<String>>,
    /// Marks the conversation read.
    #[prop(optional)]
    on_read: Option<Callback<String>>,
    /// The screen's own trailing control, such as the mute toggle.
    #[prop(optional)]
    after: Option<ChildrenFn>,
    /// The caller's rendering of the newest status.
    #[prop(optional)]
    status: Option<ChildrenFn>,
) -> impl IntoView {
    let names = participants(&conversation);
    let unread = conversation.unread;
    let target = StoredValue::new(conversation.id.clone());

    view! {
        <li class="flex items-start gap-3 p-3">
            // The address is a placeholder: a conversation is not a route yet, so the press
            // is the whole interaction and the browser's default is cancelled when it runs.
            <a
                class="min-w-0 flex-1"
                href="#"
                on:click=move |event| open_conversation(&event, target, on_open)
            >
                <p class="flex items-center gap-2 text-sm font-medium">
                    <span class="truncate">{names}</span>
                    <Show when=move || unread>
                        <span class="badge badge-primary badge-xs">"new"</span>
                    </Show>
                    <Show when=move || muted>
                        <span class="badge badge-ghost badge-xs">"muted"</span>
                    </Show>
                </p>
                // The box is painted whether or not there is a status to put in it: a
                // conversation that gains its first message keeps the row's height instead
                // of growing one line while the reader is looking at it.
                <div class="min-h-4">{status.map(|children| children())}</div>
                <p class="text-xs opacity-50">{age}</p>
            </a>
            <button
                class="btn btn-ghost btn-xs"
                disabled=move || on_read.is_none()
                on:click=move |_| {
                    if let Some(on_read) = on_read {
                        on_read.run(target.get_value());
                    }
                }
            >
                "Mark as read"
            </button>
            {after.map(|children| children())}
        </li>
    }
}

/// Opens the conversation inside the app when there is a callback for it.
///
/// Without one nothing happens, so the placeholder address is never followed. The non-wasm
/// build keeps a use of `event` so the listener argument is not reported as unused on the
/// target that has no `prevent_default`.
fn open_conversation(
    event: &leptos::ev::MouseEvent,
    id: StoredValue<String>,
    on_open: Option<Callback<String>>,
) {
    #[cfg(not(target_arch = "wasm32"))]
    let _ = event;
    if let Some(on_open) = on_open {
        #[cfg(target_arch = "wasm32")]
        event.prevent_default();
        on_open.run(id.get_value());
    }
}
