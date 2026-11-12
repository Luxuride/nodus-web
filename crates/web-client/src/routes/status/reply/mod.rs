//! The reply box under a thread: the normal composer, with the parent already chosen.

use leptos::prelude::*;
use nodus_client_core::model::Status;

use crate::routes::compose::inline::InlineComposer;

/// Writes a reply, and hands the posts back to the thread that is already on screen.
#[component]
pub(crate) fn ReplyComposer(parent: String, replies: RwSignal<Vec<Status>>) -> impl IntoView {
    let on_posted = Callback::new(move |posted: Vec<Status>| {
        replies.update(|replies| replies.extend(posted));
    });
    view! {
        <InlineComposer
            reply_to=Some(parent)
            on_posted=Some(on_posted)
            heading="Reply"
        />
    }
}
