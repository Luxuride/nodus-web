//! Search across accounts, posts and hashtags.
//!
//! The screen is `nodus_view::pages::search`: the form, its progress, its refusal, and the
//! three lists. What stays here is what that crate may not hold — the page store this screen
//! reads, the transport that answers the ask, the clock that ages a post, and the card.
//!
//! The query is the key of the answer, not the answer itself: the store keeps one page per
//! question, so pressing Search loads that page and coming back to the same question paints
//! what was already found. The field and the asked query are separate signals because typing
//! is not asking — a result list changes when a search answers, not on every keystroke.
//!
//! A hashtag or mention chip links here with `?q=`, so the question arrives with the click;
//! asking the reader to retype it would make the chip decorative.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::Status;
use nodus_sdk::page::{PageItems, PageKind};
use nodus_view::pages::search::SearchPage;

use crate::router;
use crate::routes::format::relative_time;
use crate::routes::status::StatusCard;
use crate::state::alive::Alive;
use crate::state::bridge::{pages, revision};

/// The search screen.
#[component]
pub fn Search() -> impl IntoView {
    let revision = revision();
    let seed = router::query_param("q").unwrap_or_default();
    let (query, set_query) = signal(seed.clone());
    // The last question the store answered. Kept apart from `query` so the previous answer
    // stays on screen while the next request is in flight, the way it did before the screen
    // moved into the view crate.
    let (asked, set_asked) = signal(String::new());
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(None::<String>);
    let alive = Alive::new();

    let found = Signal::derive(move || {
        revision.get();
        let text = asked.get();
        let text = text.trim();
        if text.is_empty() {
            return None;
        }
        match pages::read(&PageKind::Search(text.to_owned()))?.items {
            PageItems::Search(results) => Some(*results),
            _ => None,
        }
    });
    let accounts =
        Signal::derive(move || found.get().map(|found| found.accounts).unwrap_or_default());
    let statuses =
        Signal::derive(move || found.get().map(|found| found.statuses).unwrap_or_default());
    let tags = Signal::derive(move || found.get().map(|found| found.hashtags).unwrap_or_default());
    // Held is what separates "nothing matched" from "nothing asked": an answered question
    // leaves a page in the store even when every list in it is empty.
    let searched = Signal::derive(move || {
        revision.get();
        let text = asked.get();
        let text = text.trim();
        !text.is_empty() && pages::held(&PageKind::Search(text.to_owned()))
    });

    let ask = Callback::new(move |()| {
        let text = query.get_untracked();
        let text = text.trim().to_owned();
        if text.is_empty() {
            return;
        }
        set_loading.set(true);
        set_error.set(None);
        spawn_local(async move {
            match pages::ensure(PageKind::Search(text.clone())).await {
                Ok(_) => alive.if_alive(|| set_asked.set(text)),
                Err(message) => alive.if_alive(|| set_error.set(Some(message))),
            }
            alive.if_alive(|| set_loading.set(false));
        });
    });

    // The question arrived with the click, so it is asked once on mount rather than left in
    // the field for the reader to submit again.
    if !seed.trim().is_empty() {
        ask.run(());
    }

    let on_query = Callback::new(move |text: String| set_query.set(text));
    let on_search = ask;
    let age_of = Callback::new(move |created_at: String| relative_time(&created_at));
    let card =
        Callback::new(move |status: Status| view! { <StatusCard status=status/> }.into_any());

    view! {
        <SearchPage
            query=query.into()
            accounts=accounts
            statuses=statuses
            tags=tags
            searched=searched
            loading=loading.into()
            error=error.into()
            on_query=on_query
            on_search=on_search
            age_of=Some(age_of)
            card=Some(card)
        />
    }
}
