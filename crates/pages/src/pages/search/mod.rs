//! The search screen: the form that asks, and the three lists that answer.
//!
//! The query and the three answers are pointed at, never looked up: the search is a request,
//! and the request belongs to whoever has a transport. What is drawn here is the shape of the
//! answer — the form, its progress, its refusal, and the sections underneath, which live in
//! [`results`] because each is a full card layout.
//!
//! Post cards are the caller's, since only it knows the age of a post, the addresses it opens
//! and the verbs it offers; they arrive as a call for one status, and without one the shared
//! read-only card is drawn in the same boxes.

mod results;

pub use results::Results;

use leptos::prelude::*;
use nodus_client_core::model::{Account, Status, Tag};

use crate::{PageHeader, Screen};

/// The search screen.
#[component]
pub fn SearchPage(
    /// What the reader has typed into the search.
    query: Signal<String>,
    /// The accounts the last search found.
    accounts: Signal<Vec<Account>>,
    /// The posts the last search found.
    statuses: Signal<Vec<Status>>,
    /// The hashtags the last search found.
    tags: Signal<Vec<Tag>>,
    /// Whether a search has answered yet.
    searched: Signal<bool>,
    /// Whether a search is on its way.
    loading: Signal<bool>,
    /// The last refusal, shown above the results.
    error: Signal<Option<String>>,
    /// What the reader typed into the search.
    #[prop(optional)]
    on_query: Option<Callback<String>>,
    /// Asks the question. Absent where there is nowhere to ask it, and the button then reads
    /// the same and does nothing.
    #[prop(optional)]
    on_search: Option<Callback<()>>,
    /// Formats a timestamp as an age, for the card this page draws itself. Absent when the
    /// caller has no clock to lend, and the card then carries no age rather than a made up
    /// one.
    age_of: Option<Callback<String, String>>,
    /// The client's own status card, drawn in place of the shared read-only one.
    card: Option<Callback<Status, AnyView>>,
) -> impl IntoView {
    view! {
        <Screen>
            <PageHeader
                title="Search"
                description="Accounts, posts and hashtags, including ones this instance does not host."
            />
            <form
                class="flex items-center gap-2"
                on:submit=move |event| {
                    stop_submit(&event);
                    if let Some(on_search) = on_search {
                        on_search.run(());
                    }
                }
            >
                <input
                    type="search"
                    class="input input-bordered w-full"
                    placeholder="Search accounts, posts, hashtags…"
                    prop:value=move || query.get()
                    disabled=on_query.is_none()
                    on:input=move |event| {
                        if let Some(on_query) = on_query {
                            on_query.run(event_target_value(&event));
                        }
                    }
                />
                <button
                    class="btn btn-primary btn-sm"
                    type="submit"
                    disabled=move || loading.get() || on_search.is_none()
                >
                    {move || if loading.get() { "Searching…" } else { "Search" }}
                </button>
            </form>

            <Show when=move || error.get().is_some()>
                <p class="text-error text-xs">{move || error.get().unwrap_or_default()}</p>
            </Show>

            <Results
                accounts=accounts
                statuses=statuses
                tags=tags
                searched=searched
                age_of=age_of
                card=card
            />
        </Screen>
    }
}

/// Keeps a submit from reloading the page, on the target that has a page to reload.
///
/// The server render never runs a listener, and its event type has no `prevent_default`; the
/// non-wasm build keeps a use of `event` so the argument is not reported as unused.
fn stop_submit(event: &leptos::ev::SubmitEvent) {
    #[cfg(not(target_arch = "wasm32"))]
    let _ = event;
    #[cfg(target_arch = "wasm32")]
    event.prevent_default();
}
