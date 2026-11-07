//! The action row under a status.
//!
//! Each button runs one verb and hands the answer back to the card. There is no
//! optimistic update: a favourite that the server refused would leave the button lit
//! with nothing to undo it (the same reason the SDK dispatches consent only after a
//! successful request).

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::Status;

use super::menu::StatusMenu;
use crate::router::{self, Route};
use crate::state::alive::Alive;
use crate::state::bridge::actions;

/// Counts on the left, actions on the right.
#[component]
pub fn CardFooter(
    shown: ReadSignal<Status>,
    set_shown: WriteSignal<Status>,
    busy: ReadSignal<bool>,
    set_busy: WriteSignal<bool>,
    set_error: WriteSignal<Option<String>>,
) -> impl IntoView {
    let alive = Alive::new();

    view! {
        <footer class="flex items-center justify-between gap-2 border-t border-base-200 pt-2">
            <span class="text-xs opacity-60">
                {move || {
                    let status = shown.get();
                    format!(
                        "{} boosts · {} favourites · {} replies",
                        status.reblogs_count, status.favourites_count, status.replies_count,
                    )
                }}
            </span>
            <div class="flex items-center gap-1">
                <a
                    class="btn btn-ghost btn-sm"
                    href=move || Route::Thread(shown.get_untracked().id).path()
                    on:click=move |event| {
                        router::link(event, Route::Thread(shown.get_untracked().id));
                    }
                >
                    "Reply"
                </a>
                <Show when=move || shown.get().edited_at.is_some()>
                    <span class="text-xs opacity-50">"edited"</span>
                </Show>
                <StatusMenu shown=shown set_error=set_error/>
                <button
                    class="btn btn-ghost btn-sm"
                    class:btn-active=move || shown.get().reblogged
                    disabled=move || busy.get()
                    on:click=move |_| {
                        let id = shown.get_untracked().id;
                        let want = !shown.get_untracked().reblogged;
                        set_busy.set(true);
                        set_error.set(None);

                        spawn_local(async move {
                            match actions::reblog(&id, want).await {
                                Ok(status) => alive.if_alive(|| set_shown.set(status)),
                                Err(message) => alive.if_alive(|| set_error.set(Some(message))),
                            }
                            alive.if_alive(|| set_busy.set(false));
                        });
                    }
                >
                    "Boost"
                </button>
                <button
                    class="btn btn-ghost btn-sm"
                    class:btn-active=move || shown.get().favourited
                    disabled=move || busy.get()
                    on:click=move |_| {
                        let id = shown.get_untracked().id;
                        let want = !shown.get_untracked().favourited;
                        set_busy.set(true);
                        set_error.set(None);

                        spawn_local(async move {
                            match actions::favourite(&id, want).await {
                                Ok(status) => alive.if_alive(|| set_shown.set(status)),
                                Err(message) => alive.if_alive(|| set_error.set(Some(message))),
                            }
                            alive.if_alive(|| set_busy.set(false));
                        });
                    }
                >
                    "Favourite"
                </button>
                <button
                    class="btn btn-ghost btn-sm"
                    class:btn-active=move || shown.get().bookmarked
                    disabled=move || busy.get()
                    on:click=move |_| {
                        let id = shown.get_untracked().id;
                        let want = !shown.get_untracked().bookmarked;
                        set_busy.set(true);
                        set_error.set(None);

                        spawn_local(async move {
                            match actions::bookmark(&id, want).await {
                                Ok(status) => alive.if_alive(|| set_shown.set(status)),
                                Err(message) => alive.if_alive(|| set_error.set(Some(message))),
                            }
                            alive.if_alive(|| set_busy.set(false));
                        });
                    }
                >
                    "Bookmark"
                </button>
            </div>
        </footer>
    }
}
