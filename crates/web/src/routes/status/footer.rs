//! The action row under a status.
//!
//! Each button runs one verb and hands the answer back to the card. There is no
//! optimistic update: a favourite that the server refused would leave the button lit
//! with nothing to undo it (the same reason the SDK dispatches consent only after a
//! successful request).

use leptos::prelude::*;

use crate::ui::icons::Icon;
use leptos::task::spawn_local;
use nodus_instance::model::Status;

use super::menu::StatusMenu;
use super::share::ShareButton;
use crate::router;
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
                    format!("{} replies", status.replies_count)
                }}
            </span>
            <div class="flex items-center gap-1">
                <a
                    class="btn btn-ghost btn-sm"
                    href=move || super::thread_route(&shown.get_untracked().id).path()
                    on:click=move |event| {
                        router::link(event, super::thread_route(&shown.get_untracked().id));
                    }
                    title="Reply to this post"
                >
                    <Icon icon=Icon::Reply/>
                    <span class="ml-1">"Reply"</span>
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
                    title=move || if shown.get().reblogged { "Undo boost" } else { "Boost to your followers" }
                >
                    <Icon icon=Icon::Repeat/>
                    <span class="ml-1 tabular-nums">{move || shown.get().reblogs_count}</span>
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
                    title=move || if shown.get().favourited { "Remove from favourites" } else { "Add to favourites" }
                >
                    <Icon icon=Icon::Star/>
                    <span class="ml-1 tabular-nums">{move || shown.get().favourites_count}</span>
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
                    title=move || if shown.get().bookmarked { "Remove from bookmarks" } else { "Save for later" }
                >
                    <Icon icon=Icon::Bookmark/>
                </button>
                <ShareButton shown=shown set_error=set_error/>
            </div>
        </footer>
    }
}
