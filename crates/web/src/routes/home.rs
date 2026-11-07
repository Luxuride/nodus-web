//! The home, local and federated timelines, rendered out of SDK state.
//!
//! The page owns no list. It reads what the SDK holds, asks it to fill what is
//! missing or stale, and continues the same cursor when the reader asks for more —
//! so returning to this screen paints immediately instead of starting from a
//! spinner.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_sdk::Timeline;

use crate::routes::status::StatusCard;
use crate::state::alive::Alive;
use crate::state::bridge::{revision, timeline};

/// Which timeline the tabs ask for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Home,
    Local,
    Federated,
}

impl Tab {
    fn label(self) -> &'static str {
        match self {
            Self::Home => "Home",
            Self::Local => "Local",
            Self::Federated => "Federated",
        }
    }

    fn timeline(self) -> Timeline {
        match self {
            Self::Home => Timeline::Home,
            Self::Local => Timeline::Local,
            Self::Federated => Timeline::Federated,
        }
    }
}

/// A timeline with its tabs.
#[component]
pub fn Home() -> impl IntoView {
    let revision = revision();
    let (tab, set_tab) = signal(Tab::Home);
    // A request is in flight. Starts true because the first thing that happens is
    // that request: "empty" before it is a guess, not a fact.
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);
    let alive = Alive::new();

    let which = move || tab.get().timeline();
    let held = move || {
        revision.get();
        timeline::read(which(), None)
    };

    // Runs on mount and on every tab change. `ensure` answers from state when the
    // page is inside its TTL, so switching tabs costs nothing after the first visit.
    Effect::new(move |_| {
        let which = which();
        set_loading.set(true);
        set_error.set(None);
        spawn_local(async move {
            match timeline::ensure(which, None, None).await {
                Ok(_) => alive.if_alive(|| set_loading.set(false)),
                Err(message) => alive.if_alive(|| {
                    set_error.set(Some(message));
                    set_loading.set(false);
                }),
            }
        });
    });

    let load_more = move || {
        let which = which();
        let Some(cursor) = held().1 else {
            return;
        };
        set_loading.set(true);
        set_error.set(None);
        spawn_local(async move {
            match timeline::ensure(which, None, Some(cursor)).await {
                Ok(_) => alive.if_alive(|| set_loading.set(false)),
                Err(message) => alive.if_alive(|| {
                    set_error.set(Some(message));
                    set_loading.set(false);
                }),
            }
        });
    };

    view! {
        <div class="flex flex-col gap-3">
            <div role="tablist" class="tabs tabs-box w-fit">
                <For
                    each=|| [Tab::Home, Tab::Local, Tab::Federated]
                    key=|tab| format!("{tab:?}")
                    let:kind
                >
                    <button
                        role="tab"
                        class="tab"
                        class:tab-active=move || tab.get() == kind
                        on:click=move |_| set_tab.set(kind)
                    >
                        {kind.label()}
                    </button>
                </For>
            </div>

            {move || error.get().map(|message| view! {
                <div class="alert alert-error py-2 text-sm" role="alert">
                    <span>{message}</span>
                </div>
            })}

            // Unknown is not empty: the note below appears only once a load has
            // finished and still found nothing.
            <Show when=move || loading.get() && held().0.is_empty()>
                <div class="flex justify-center py-6">
                    <span class="loading loading-spinner loading-md"></span>
                </div>
            </Show>

            <For
                each=move || held().0
                key=|status| status.id.clone()
                let:status
            >
                <StatusCard status=status/>
            </For>

            <Show when=move || {
                !loading.get() && held().0.is_empty() && error.get().is_none()
            }>
                <div class="card border border-base-300 bg-base-100">
                    <div class="card-body items-center p-6 text-center">
                        <h2 class="card-title text-base">"Nothing here yet"</h2>
                        <p class="text-sm opacity-70">
                            "This timeline is empty. Following a few accounts fills it."
                        </p>
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
        </div>
    }
}
