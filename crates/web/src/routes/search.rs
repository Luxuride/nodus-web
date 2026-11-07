//! Search across accounts, posts and hashtags.
//!
//! `resolve = true` is sent deliberately: without it an instance only answers from what
//! it has already seen, so searching for a handle you have never interacted with returns
//! nothing and looks broken rather than slow.
//!
//! The three result lists are three signals, not one `SearchResults`: each one is rendered
//! by a list that owns its copy, and one signal per list is what keeps them from fighting
//! over the same value.

use leptos::prelude::*;

use crate::router;
use crate::ui::page::PageHeader;
use leptos::task::spawn_local;
use nodus_instance::model::{Account, Status, Tag};

use crate::state::alive::Alive;
use crate::state::bridge::actions;

mod results;

use self::results::Results;

/// The search screen.
#[component]
pub fn Search() -> impl IntoView {
    let (query, set_query) = signal(String::new());
    let (accounts, set_accounts) = signal(Vec::<Account>::new());
    let (statuses, set_statuses) = signal(Vec::<Status>::new());
    let (tags, set_tags) = signal(Vec::<Tag>::new());
    let (searched, set_searched) = signal(false);
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(None::<String>);

    let run = move |()| {
        let text = query.get_untracked();
        if text.trim().is_empty() {
            return;
        }
        let alive = Alive::new();
        set_loading.set(true);
        set_error.set(None);
        spawn_local(async move {
            match actions::search(text.trim(), true).await {
                Ok(found) => {
                    alive.if_alive(|| {
                        set_accounts.set(found.accounts);
                        set_statuses.set(found.statuses);
                        set_tags.set(found.hashtags);
                        set_searched.set(true);
                    });
                }
                Err(message) => alive.if_alive(|| set_error.set(Some(message))),
            }
            alive.if_alive(|| set_loading.set(false));
        });
    };

    // A hashtag or mention chip links here with `?q=`, so the question arrives with the
    // click; asking the reader to retype it would make the chip decorative.
    if let Some(seed) = router::query_param("q") {
        if query.get_untracked().is_empty() {
            set_query.set(seed);
            run(());
        }
    }

    view! {
        <section class="flex flex-col gap-3">
            <PageHeader title="Search" description="Accounts, posts and hashtags, including ones this instance does not host."/>
            <form
                class="flex items-center gap-2"
                on:submit=move |event| {
                    event.prevent_default();
                    run(());
                }
            >
                <input
                    type="search"
                    class="input input-bordered w-full"
                    placeholder="Search accounts, posts, hashtags…"
                    prop:value=move || query.get()
                    on:input=move |event| set_query.set(event_target_value(&event))
                />
                <button
                    class="btn btn-primary btn-sm"
                    type="submit"
                    disabled=move || loading.get()
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
            />
        </section>
    }
}
