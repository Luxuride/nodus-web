//! The composer that sits above the home timeline.
//!
//! Always open, never toggled: writing is the reason the home timeline exists, and a
//! button that expands a form is one decision more than a box that is simply there. It
//! is deliberately the *short* path — text, media, visibility, and a link to the full
//! composer for anything else (scheduling, a content warning, a reply's thread context).

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::{MediaAttachment, Visibility};

use crate::routes::compose::attachments::Attachments;
use crate::state::alive::Alive;
use crate::state::bridge::{actions, timeline};

/// A box above the timeline: write, attach, post.
#[component]
pub fn InlineComposer(
    /// The timeline to refresh after a post, so the new status is visible at once.
    which: StoredValue<nodus_sdk::Timeline>,
    /// Bumped to ask the timeline to reload.
    set_reload: WriteSignal<u32>,
) -> impl IntoView {
    let (text, set_text) = signal(String::new());
    let (visibility, set_visibility) = signal(Visibility::Public);
    let (busy, set_busy) = signal(false);
    let (error, set_error) = signal(None::<String>);
    let media = RwSignal::new(Vec::<MediaAttachment>::new());
    let alive = Alive::new();

    let post = move |()| {
        if busy.get_untracked() {
            return;
        }
        let body = text.get_untracked();
        if body.trim().is_empty() {
            return;
        }
        let mut draft = actions::NewStatus::text(body, visibility.get_untracked());
        draft.media_ids = media
            .get_untracked()
            .iter()
            .map(|item| item.id.clone())
            .collect();

        set_busy.set(true);
        set_error.set(None);
        spawn_local(async move {
            match actions::publish(&draft).await {
                Ok(_) => {
                    // The draft is gone once the instance has it; nothing is kept for a
                    // retry, because a retry after a success is a duplicate post.
                    if alive.is_alive() {
                        set_text.set(String::new());
                        media.set(Vec::new());
                        let _ = timeline::refresh(which.get_value(), None).await;
                        alive.if_alive(|| {
                            set_busy.set(false);
                            set_reload.update(|generation| *generation += 1);
                        });
                    }
                }
                Err(message) => alive.if_alive(|| {
                    set_error.set(Some(message));
                    set_busy.set(false);
                }),
            }
        });
    };

    let on_submit = Callback::new(post);
    let count = move || text.get().chars().count();

    view! {
        <div class="card border border-base-300 bg-base-100">
            <div class="card-body gap-2 p-3">
                <textarea
                    class="textarea textarea-ghost min-h-20 w-full resize-none p-1 text-[15px] focus:bg-transparent focus:outline-none"
                    placeholder="What's on your mind?"
                    aria-label="Write a post"
                    prop:value=text
                    on:input=move |event| set_text.set(event_target_value(&event))
                />

                <Attachments media=media set_error=set_error/>

                <div class="flex flex-wrap items-center gap-2">
                    <VisibilityPicker visibility=visibility set_visibility=set_visibility/>

                    <span class="text-xs tabular-nums opacity-50" class=("text-error", move || count() > 500)>
                        {count}
                    </span>

                    <a
                        class="btn btn-ghost btn-sm ml-auto gap-1"
                        href=crate::router::Route::Compose.path()
                        title="The full composer: content warning, scheduling, replies"
                    >
                        "More"
                    </a>

                    <button
                        class="btn btn-primary btn-sm gap-1"
                        disabled=move || busy.get() || text.get().trim().is_empty()
                        on:click=move |_| on_submit.run(())
                    >
                        {move || if busy.get() { "Posting…" } else { "Post" }}
                    </button>
                </div>

                {move || error.get().map(|message| view! {
                    <p class="text-error text-xs" role="alert">{message}</p>
                })}
            </div>
        </div>
    }
}

/// The four visibilities, as a select.
#[component]
fn VisibilityPicker(
    visibility: ReadSignal<Visibility>,
    set_visibility: WriteSignal<Visibility>,
) -> impl IntoView {
    view! {
        <select
            class="select select-ghost select-sm w-fit"
            aria-label="Who can see this post"
            on:change=move |event| {
                let value = event_target_value(&event);
                set_visibility.set(match value.as_str() {
                    "unlisted" => Visibility::Unlisted,
                    "private" => Visibility::Private,
                    "direct" => Visibility::Direct,
                    _ => Visibility::Public,
                });
            }
        >
            <For
                each=|| [
                    (Visibility::Public, "Public"),
                    (Visibility::Unlisted, "Unlisted"),
                    (Visibility::Private, "Followers"),
                    (Visibility::Direct, "Direct"),
                ]
                key=|(visibility, _)| format!("{visibility:?}")
                let:entry
            >
                <option value=entry.0.as_str() selected=move || visibility.get() == entry.0>
                    {entry.1}
                </option>
            </For>
        </select>
    }
}
