//! People the instance suggests.
//!
//! Read out of the SDK's page store, like every other list: opening this screen again
//! paints what was already fetched, and only a page older than the store's TTL asks the
//! instance for more. The directory is the one list Mastodon pages by offset, so its
//! cursor is a count — the store treats it as an opaque string either way.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_sdk::page::PageKind;

use self::card::PersonCard;
use crate::state::alive::Alive;
use crate::state::bridge::{pages, revision};
use crate::ui::page::PageHeader;

mod card;

/// Local or federated accounts, in the instance's own order of activity.
#[component]
pub fn Directory() -> impl IntoView {
    let revision = revision();
    let (local_only, set_local_only) = signal(true);
    // Starts true: the first thing this screen does is ask, so "empty" before the answer
    // would be a guess.
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);
    let alive = Alive::new();

    let kind = move || PageKind::Directory(local_only.get());
    let held = move || {
        revision.get();
        pages::accounts(&kind())
    };
    let people = move || held().map(|(list, _)| list).unwrap_or_default();
    let cursor = move || held().and_then(|(_, cursor)| cursor);

    let load = move |after: Option<String>| {
        let kind = kind();
        spawn_local(async move {
            match pages::ensure_after(kind, after).await {
                Ok(_) => alive.if_alive(|| set_loading.set(false)),
                Err(message) => alive.if_alive(|| {
                    set_error.set(Some(message));
                    set_loading.set(false);
                }),
            }
        });
    };

    // One effect for both scopes: switching asks the store for the other key, which is
    // either already held or fetched once.
    Effect::new(move |_| {
        let _ = local_only.get();
        load(None);
    });

    let more = move |()| load(cursor());

    view! {
        <nodus_view::Screen>
            <PageHeader
                title="People"
                description="Accounts this instance suggests, most active first."
            />

            <div role="tablist" class="tabs tabs-box w-fit">
                <button
                    role="tab"
                    class="tab"
                    class:tab-active=move || local_only.get()
                    on:click=move |_| set_local_only.set(true)
                >
                    "This instance"
                </button>
                <button
                    role="tab"
                    class="tab"
                    class:tab-active=move || !local_only.get()
                    on:click=move |_| set_local_only.set(false)
                >
                    "Known elsewhere"
                </button>
            </div>

            {move || error.get().map(|message| view! {
                <div class="alert alert-error py-2 text-sm" role="alert">
                    <span>{message}</span>
                </div>
            })}

            <Show when=move || loading.get() && people().is_empty()>
                <div class="flex justify-center py-6">
                    <span class="loading loading-spinner loading-md"></span>
                </div>
            </Show>

            <ul class="flex flex-col gap-1">
                <For each=people key=|account| account.id.clone() let:account>
                    <PersonCard account=account/>
                </For>
            </ul>

            <Show when=move || !loading.get() && people().is_empty()>
                <div class="card border border-base-300 bg-base-100">
                    <div class="card-body items-center p-6 text-center">
                        <h2 class="card-title text-base">"Nobody to suggest yet"</h2>
                        <p class="text-sm opacity-70">
                            "This instance has not published a directory of accounts."
                        </p>
                    </div>
                </div>
            </Show>

            <Show when=move || cursor().is_some()>
                <button
                    class="btn btn-ghost btn-sm w-full"
                    disabled=move || loading.get()
                    on:click=move |_| more(())
                >
                    {move || if loading.get() { "Loading…" } else { "Load more" }}
                </button>
            </Show>
        </nodus_view::Screen>
    }
}
