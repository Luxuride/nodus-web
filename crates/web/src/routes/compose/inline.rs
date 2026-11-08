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
use crate::routes::compose::inline::bar::ComposerBar;
use crate::state::alive::Alive;
use crate::state::bridge::{actions, timeline};

mod bar;

/// How long a post may be. Mastodon's default; an instance may configure another, and the
/// counter says "the instance refuses" rather than pretending this number is a law.
pub(super) const LIMIT: usize = 500;

/// When a post should go out.
///
/// Presets, not a date picker: this box is the short path, and an instant the reader has
/// to assemble is the long one. Every preset is computed in the browser's own zone, which
/// is the zone the reader is looking at the clock in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum When {
    Now,
    In30,
    In2h,
    Tomorrow,
}

impl When {
    pub(super) const ALL: [Self; 4] = [Self::Now, Self::In30, Self::In2h, Self::Tomorrow];

    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::Now => "Send now",
            Self::In30 => "In 30 minutes",
            Self::In2h => "In 2 hours",
            Self::Tomorrow => "Tomorrow at 9",
        }
    }

    /// An RFC-3339 instant, or `None` for "send it now".
    pub(super) fn at(self) -> Option<String> {
        let start = match self {
            Self::Now => return None,
            Self::In30 => js_sys::Date::now() + 1_800_000.0,
            Self::In2h => js_sys::Date::now() + 7_200_000.0,
            Self::Tomorrow => {
                let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(js_sys::Date::now()));
                date.set_hours(9);
                date.set_minutes(0);
                date.set_seconds(0);
                date.set_milliseconds(0);
                // It is already past nine: the reader meant tomorrow, not a minute ago.
                if date.get_time() <= js_sys::Date::now() {
                    date.set_date(date.get_date() + 1);
                }
                date.get_time()
            }
        };
        let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(start));
        Some(date.to_iso_string().into())
    }
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
    let (when, set_when) = signal(When::Now);
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
        let target = when.get_untracked();
        let at = target.at();

        set_busy.set(true);
        set_error.set(None);
        set_notice.set(None);
        spawn_local(async move {
            let outcome = match at {
                // Scheduling sends the text only: the queue is a list of drafts on the
                // instance, so the media ids travel with the request that creates it.
                Some(at) => actions::schedule(&draft.text, chosen, &at)
                    .await
                    .map(|()| format!("Scheduled: {}", target.label().to_lowercase())),
                None => actions::publish(&draft).await.map(|_| String::new()),
            };
            match outcome {
                Ok(scheduled) => {
                    // The draft is gone once the instance has it; nothing is kept for a
                    // retry, because a retry after a success is a duplicate post.
                    if alive.is_alive() {
                        set_text.set(String::new());
                        media.set(Vec::new());
                        set_when.set(When::Now);
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
            class="card border-2 border-base-300 bg-base-100 shadow-sm focus-within:border-primary/50"
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

                <Attachments media=media set_error=set_error/>

                <ComposerBar
                    text=text
                    visibility=visibility
                    set_visibility=set_visibility
                    when=when
                    set_when=set_when
                    busy=busy
                    on_submit=on_submit
                />

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
            class="select select-bordered select-xs w-fit"
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
