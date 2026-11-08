//! The composer that sits above the home timeline.
//!
//! Always open, never toggled: writing is the reason the home timeline exists, and a
//! button that expands a form is one decision more than a box that is simply there. It
//! is deliberately the *short* path — text, media, visibility, and a link to the full
//! composer for anything else (scheduling, a content warning, a reply's thread context).

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::{MediaAttachment, Visibility};

use crate::routes::compose::attachments::{self, Attachments};
use crate::routes::compose::inline::bar::ComposerBar;
use crate::state::alive::Alive;
use crate::state::bridge::{actions, timeline};

mod bar;
mod schedule;

use bar::Notice;
use schedule::SchedulePanel;

/// How long a post may be. Mastodon's default; an instance may configure another, and the
/// counter says "the instance refuses" rather than pretending this number is a law.
pub(super) const LIMIT: usize = 500;

/// A `datetime-local` value as an RFC-3339 instant, or `None` when it is empty or not a
/// date the browser understands — the caller then tells the reader to pick one instead of
/// posting at a guessed time.
fn custom_time(value: &str) -> Option<String> {
    if value.trim().is_empty() {
        return None;
    }
    let parsed = js_sys::Date::new(&wasm_bindgen::JsValue::from_str(value));
    let millis = parsed.get_time();
    if millis.is_nan() {
        return None;
    }
    Some(parsed.to_iso_string().into())
}

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
    // `YYYY-MM-DDTHH:mm` as the input reports it, in this browser's zone. Empty means
    // "send it now": scheduling is an exception, not the default path.
    let (custom, set_custom) = signal(String::new());
    let (notice, set_notice) = signal(None::<String>);
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

        let chosen = visibility.get_untracked();
        let custom = custom.get_untracked();
        let at = custom_time(&custom);
        if !custom.is_empty() && at.is_none() {
            set_error.set(Some("That is not a date and time this browser can read.".to_owned()));
            return;
        }

        set_busy.set(true);
        set_error.set(None);
        set_notice.set(None);
        spawn_local(async move {
            let outcome = match at {
                // Scheduling sends the text only: the queue is a list of drafts on the
                // instance, so the media ids travel with the request that creates it.
                Some(at) => actions::schedule(&draft.text, chosen, &at)
                    .await
                    .map(|()| format!("Scheduled: {}", Notice::of(&custom))),
                None => actions::publish(&draft).await.map(|_| String::new()),
            };
            match outcome {
                Ok(scheduled) => {
                    // The draft is gone once the instance has it; nothing is kept for a
                    // retry, because a retry after a success is a duplicate post.
                    if alive.is_alive() {
                        set_text.set(String::new());
                        media.set(Vec::new());
                        set_custom.set(String::new());
                        // A scheduled post is not in the timeline yet, so refreshing it
                        // would fetch the same page and change nothing.
                        if scheduled.is_empty() {
                            let _ = timeline::refresh(which.get_value(), None).await;
                        }
                        alive.if_alive(|| {
                            set_notice.set(Some(scheduled.clone()).filter(|line| !line.is_empty()));
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

    view! {
        <section
            class="card border-2 border-base-300 bg-base-100 shadow-sm has-[textarea:focus]:border-primary/50"
            aria-label="New post"
        >
            <div class="card-body gap-2 p-3">
                <h2 class="text-xs font-semibold uppercase tracking-wide opacity-50">"New post"</h2>
                <textarea
                    class="textarea textarea-ghost min-h-20 w-full resize-none p-1 text-[15px] focus:bg-transparent focus:outline-none"
                    placeholder="What's on your mind?"
                    aria-label="Write a post"
                    prop:value=text
                    on:input=move |event| set_text.set(event_target_value(&event))
                />

                <ComposerBar
                    text=text
                    at=custom
                    set_at=set_custom
                    visibility=visibility
                    set_visibility=set_visibility
                    busy=busy
                    on_add_media=Callback::new(move |()| attachments::request_pick())
                    on_submit=on_submit
                />

                // The thumbnails get their own row: they are the result of the control
                // above, and squeezing them into it would push the buttons around.
                <Attachments media=media set_error=set_error/>

                {move || error.get().map(|message| view! {
                    <p class="text-error text-xs" role="alert">{message}</p>
                })}

                {move || notice.get().map(|line| view! {
                    <p class="text-xs text-success">{line}</p>
                })}
            </div>
        </section>
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
            class="select select-bordered select-sm w-fit"
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
