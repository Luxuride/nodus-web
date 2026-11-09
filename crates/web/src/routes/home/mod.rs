//! One timeline, rendered out of SDK state.
//!
//! Each feed is its own address (`/home`, `/local`, `/federated`) rather than a tab on a
//! shared page: a tab cannot be linked to, cannot be a rail destination, and cannot be
//! hidden when the reader does not want it. The page owns no list — it reads what the
//! SDK holds, asks it to fill what is missing or stale, and continues the same cursor
//! when the reader asks for more.

mod banner;

use leptos::prelude::*;
use nodus_instance::model::FilterContext;

mod tabs;

use leptos::task::spawn_local;
use nodus_sdk::Timeline;
mod header;
pub use header::TimelineHeader;
pub use tabs::PrimaryTabs;

use self::banner::Banner;
use crate::routes::filtering::Filtered;
use crate::state::alive::Alive;
use crate::state::bridge::{revision, timeline};
use crate::ui::page::PageHeader;

/// The title, description and empty note for one timeline.
///
/// The three timelines differ only in what they hold, so the copy lives here rather than
/// being passed in by each route: "Nothing here yet" with "following a few accounts
/// fills it" would be wrong on the federated timeline, which fills from remote traffic.
fn copy(which: &Timeline) -> (&'static str, &'static str, &'static str) {
    match which {
        Timeline::Local => (
            "Local",
            "Posts from accounts on this instance, whether you follow them or not.",
            "Nobody on this instance has posted publicly yet. Unlisted and follower-only posts do not appear here.",
        ),
        Timeline::Federated => (
            "Federated",
            "Posts this instance has received from the accounts it knows about.",
            "This instance has not received anything from elsewhere yet.",
        ),
        _ => (
            "Home",
            "Posts from the accounts you follow, newest first.",
            "Nothing here yet. Following a few accounts fills it.",
        ),
    }
}

/// A timeline with its composer above it (home only) and a "Load more" foot.
#[component]
pub fn Home(which: Timeline) -> impl IntoView {
    // `Timeline` is a `Clone` enum (an account timeline carries an id), not a `Copy`
    // one, and every closure below captures it; a `StoredValue` hands each of them a
    // copy without turning the whole component into a `FnOnce`.
    let which = StoredValue::new(which);
    let revision = revision();
    // A request is in flight. Starts true because the first thing that happens is that
    // request: "empty" before it is a guess, not a fact.
    // Only look busy when the store has nothing: a visit to a timeline we already hold
    // paints it at once, and the refresh happens behind the content.
    let (loading, set_loading) = signal(!timeline::held(which.get_value(), None));
    let (error, set_error) = signal(None::<String>);
    // Posting from the composer asks this timeline for a fresh first page, and the
    // effect below must not fight it by loading from the cache in the same breath.
    let (reload, _set_reload) = signal(0_u32);
    let alive = Alive::new();

    let (title, description, empty_note) = copy(&which.get_value());
    let held = move || {
        revision.get();
        timeline::read(which.get_value(), None)
    };

    Effect::new(move |_| {
        // Reading `reload` joins this effect to it; the value itself is unused because
        // the load below is unconditional.
        let _ = reload.get();
        set_loading.set(!timeline::held(which.get_value(), None));
        set_error.set(None);
        spawn_local(async move {
            match timeline::ensure(which.get_value(), None, None).await {
                Ok(_) => alive.if_alive(|| set_loading.set(false)),
                Err(message) => alive.if_alive(|| {
                    set_error.set(Some(message));
                    set_loading.set(false);
                }),
            }
        });
    });

    let load_more = move || {
        let Some(cursor) = held().1 else {
            return;
        };
        set_loading.set(true);
        set_error.set(None);
        spawn_local(async move {
            match timeline::ensure(which.get_value(), None, Some(cursor)).await {
                Ok(_) => alive.if_alive(|| set_loading.set(false)),
                Err(message) => alive.if_alive(|| {
                    set_error.set(Some(message));
                    set_loading.set(false);
                }),
            }
        });
    };

    view! {
        // One width for every primary timeline: switching between Home, Feed, Local and
        // Federated must not move the column. The extra padding Feed used to carry is
        // dropped for the same reason.
        <nodus_view::Screen>
            <PageHeader title=title description=description/>

            // Above the posts, not pinned over them: a sticky composer pushed the first
            // status out of view on a phone and cost more than it saved. It scrolls with
            // the timeline like everything else.
            <TimelineHeader/>

            <Banner/>

            {move || error.get().map(|message| view! {
                <div class="alert alert-error py-2 text-sm" role="alert">
                    <span>{message}</span>
                </div>
            })}

            // Unknown is not empty: the note below appears only once a load has finished
            // and still found nothing.
            <Show when=move || loading.get() && held().0.is_empty()>
                <div class="flex justify-center py-6">
                    <span class="loading loading-spinner loading-md"></span>
                </div>
            </Show>

            // A list, not a run of bare `<li>`s: a screen reader announces the posts as
            // one list, and the cards keep their own `li` roots.
            <ul class="flex flex-col gap-2">
                <For
                    each=move || held().0
                    key=|status| status.id.clone()
                    let:status
                >
                    <Filtered status=status context=FilterContext::Home/>
                </For>
            </ul>

            <Show when=move || {
                !loading.get() && held().0.is_empty() && error.get().is_none()
            }>
                <div class="card border border-base-300 bg-base-100">
                    <div class="card-body items-center p-6 text-center">
                        <h2 class="card-title text-base">"Nothing here yet"</h2>
                        <p class="text-sm opacity-70">{empty_note}</p>
                    </div>
                </div>
            </Show>

            <Show when=move || held().1.is_some()>
                <button
                    class="btn btn-ghost btn-sm w-full"
                    disabled=move || loading.get()
                    on:click=move |_| load_more()
                >
                    {move || if loading.get() { "Loading…" } else { "Load more" }}
                </button>
            </Show>
        </nodus_view::Screen>
    }
}
