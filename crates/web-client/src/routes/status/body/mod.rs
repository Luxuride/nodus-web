//! The part of a status that is content: text, content warning, media, poll.
//!
//! The markup lives in `nodus-view` because the server paints it too. What stays here is
//! what that crate may not know: the viewer the gallery opens, the vote transport, and the
//! reader's saved preference for sensitive media, which is storage.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_client_core::model::Status;
use nodus_ui::media_grid::Picture;
use nodus_ui::status_body::StatusBody as SharedBody;

use super::sensitive;
use crate::media;
use crate::state::alive::Alive;
use crate::state::bridge::actions;

/// Content, media and poll of one status.
#[component]
pub fn StatusBody(shown: ReadSignal<Status>, set_shown: WriteSignal<Status>) -> impl IntoView {
    let (error, set_error) = signal(None::<String>);
    // The gallery position is the driver's business, so the shared grid only reports the
    // press; opening the viewer stays here.
    let on_open = Callback::new(move |(pictures, position): (Vec<Picture>, usize)| {
        media::expand(pictures, position);
    });
    let on_vote = Callback::new(move |(poll_id, choices): (String, Vec<usize>)| {
        vote(poll_id, choices, set_shown, set_error);
    });
    // Whether a veil starts open is a stored preference, so it is read once here and handed
    // to the shared body as its starting state.
    let revealed = sensitive::reveals_automatically();

    view! {
        // The body reads the whole status, so an answered verb — which replaces the card's
        // copy — repaints the counts behind the veil and the poll's tally.
        {move || view! {
            <SharedBody
                status=shown.get()
                on_open_media=on_open
                on_vote=on_vote
                revealed_init=revealed
            />
        }}
        // The poll's own error state covers an empty pick; a refused vote arrives from the
        // transport, which the shared crate cannot reach, so it is drawn here.
        <Show when=move || error.get().is_some()>
            <nodus_ui::Note class="text-error mt-1 text-xs">
                {move || error.get().unwrap_or_default()}
            </nodus_ui::Note>
        </Show>
    }
}

/// Sends the picked options and folds the answer back into the card.
///
/// The vote answers with the whole status, which is the only way to learn the new counts:
/// Mastodon has no "read this poll" endpoint, so the card replaces its copy with the
/// response instead of trusting a locally incremented number. The writes go through
/// [`Alive`] because the answer arrives after the reader may have left the screen.
fn vote(
    poll_id: String,
    choices: Vec<usize>,
    set_shown: WriteSignal<Status>,
    set_error: WriteSignal<Option<String>>,
) {
    let alive = Alive::new();
    set_error.set(None);
    spawn_local(async move {
        match actions::vote(&poll_id, &choices).await {
            Ok(status) => alive.if_alive(|| set_shown.set(status)),
            Err(message) => alive.if_alive(|| set_error.set(Some(message))),
        }
    });
}
