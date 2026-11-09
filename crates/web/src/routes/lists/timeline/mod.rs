//! The selected list's timeline.
//!
//! The panel is `nodus_view::pages::lists::ListTimeline`: the list of posts, the empty
//! sentence, the paging control and the error line. What stays here is the page store read
//! and the request that pages it; the card is this crate's, because a mounted post opens a
//! thread and offers verbs.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::Status;
use nodus_sdk::Timeline;
use nodus_view::pages::lists::ListTimeline as ListTimelinePanel;

use super::super::status::StatusCard;
use crate::state::alive::Alive;
use crate::state::bridge::{revision, timeline};

/// A list timeline, or nothing when no list is selected.
#[component]
pub fn ListTimeline(list_id: Signal<Option<String>>) -> impl IntoView {
    let changed = revision();
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(None::<String>);

    // The page lives in the SDK, like every other timeline: this panel keeps no statuses
    // of its own, so selecting the list again paints from state instead of refetching.
    let which = move || list_id.get().map(Timeline::List);
    let statuses = move || {
        changed.get();
        which()
            .map(|which| timeline::read(which, None).0)
            .unwrap_or_default()
    };
    let cursor = move || {
        changed.get();
        which().and_then(|which| timeline::read(which, None).1)
    };

    let load = move |id: String, after: Option<String>| {
        let alive = Alive::new();
        set_loading.set(true);
        set_error.set(None);
        spawn_local(async move {
            match timeline::ensure(Timeline::List(id), None, after).await {
                Ok(_) => {}
                Err(message) => alive.if_alive(|| set_error.set(Some(message))),
            }
            alive.if_alive(|| set_loading.set(false));
        });
    };

    // Selecting another list must not append to the previous one's page: the cursor
    // belongs to a list, so it is dropped with the contents.
    Effect::new(move |_| {
        if let Some(id) = list_id.get() {
            load(id, None);
        }
    });

    let more = Callback::new(move |()| {
        if let (Some(id), Some(after)) = (list_id.get_untracked(), cursor()) {
            load(id, Some(after));
        }
    });

    let card =
        Callback::new(move |status: Status| view! { <StatusCard status=status/> }.into_any());

    view! {
        <ListTimelinePanel
            list_id=list_id
            statuses=Signal::derive(statuses)
            cursor=Signal::derive(cursor)
            loading=loading.into()
            error=error.into()
            on_more=more
            card=Some(card)
        />
    }
}
