//! The composer that sits above the home timeline.
//!
//! Always open, never toggled: writing is the reason the home timeline exists, and a
//! button that expands a form is one decision more than a box that is simply there. It
//! is deliberately the *short* path — text, media, visibility, and a link to the full
//! composer for anything else (scheduling, a content warning, a reply's thread context).

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::{MediaAttachment, Visibility};
use wasm_bindgen::JsCast as _;

use crate::routes::compose::attachments::{self, Attachments};
use crate::routes::compose::inline::bar::ComposerBar;
use crate::state::alive::Alive;
use crate::state::bridge::{actions, timeline};

mod bar;
pub(crate) mod emoji;
mod emoji_toggle;
mod poll;
mod schedule;

use bar::Notice;
use emoji::Suggestions;
use poll::PollEditor;
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
    let draft = Draft::new(which, set_reload);
    // One catalogue for both controls — the suggestions popup and the picker ask the
    // instance once, not once per keystroke and once per open.
    let emoji_catalogue = emoji::catalogue();
    let (picking, set_picking) = signal(false);
    let (emoji_open, set_emoji_open) = signal(false);
    // `Draft` is not `Copy` (it owns nothing but holds the SDK handle), so the submit
    // handler gets its own copy of the `Copy` half and the rest of the view keeps `draft`.
    let submit = draft.snapshot_of_signals();
    // The completion replaces the shortcode being typed; the picker appends because no
    // token is in progress. One callback for both, so a picked emoji and a completed one
    // leave the text in the same shape.
    let text_field = draft.text;
    let emoji_insert = Callback::new(move |label: String| {
        let text = text_field.get_untracked();
        let next = emoji::pending(&text).map_or_else(
            || format!("{text}{label} "),
            |(colon, _)| emoji::complete(&text, colon, &label),
        );
        text_field.set(next);
        set_emoji_open.set(false);
        set_picking.set(false);
        focus_textarea();
    });

    // The shortcode completion follows the caret, which the editor reports as plain text
    // changes: a token is open exactly while the draft ends in one.
    Effect::new(move |_| set_emoji_open.set(emoji::pending(&draft.text.get()).is_some()));

    view! {
        <nodus_view::ComposerCard>
                <h2 class="text-xs font-semibold uppercase tracking-wide opacity-50">"New post"</h2>
                // The suggestions sit above the box, where the writer is already looking:
                // a dialog would steal focus and swallow the next keystroke.
                <Suggestions
                    query=Signal::derive(move || {
                        emoji::pending(&draft.text.get())
                            .map(|(_, query)| query)
                            .unwrap_or_default()
                    })
                    open=emoji_open
                    catalogue=emoji_catalogue
                    insert=emoji_insert
                />
                <crate::ui::editor::RichEditor
                    text=draft.text.read_only()
                    set_text=draft.text.write_only()
                    emoji=emoji_catalogue
                    placeholder="What's on your mind?"
                    on_submit=Callback::new(move |()| submit.submit())
                />

                <ComposerBar
                    text=draft.text.read_only()
                    at=draft.custom.read_only()
                    set_at=draft.custom.write_only()
                    visibility=draft.visibility.read_only()
                    set_visibility=draft.visibility.write_only()
                    busy=draft.busy.read_only()
                    on_add_media=Callback::new(move |()| attachments::request_pick())
                    // A toggle, not a latch: with a latch the control is a no-op the moment the
                    // editor is open, which is what made it read as broken.
                    emoji_open=picking
                    set_emoji_open=set_picking
                    on_add_poll=Callback::new(move |()| {
                        draft
                            .poll_open
                            .update(|open| *open = !*open);
                    })
                    language=draft.language.read_only()
                    set_language=draft.language.write_only()
                    on_submit=Callback::new(move |()| submit.submit())
                />

                <PollEditor
                    open=draft.poll_open
                    options=draft.options
                    expires_in=draft.expires_in
                    multiple=draft.multiple
                    hide_totals=draft.hide_totals
                />

                <Attachments media=draft.media set_error=draft.error.write_only()/>

                {move || draft.error.get().map(|message| view! {
                    <p class="text-error text-xs" role="alert">{message}</p>
                })}

                {move || draft.notice.get().map(|line| view! {
                    <p class="text-xs text-success">{line}</p>
                })}
        </nodus_view::ComposerCard>
    }
}

/// Everything the composer holds, so submitting is one call rather than a closure that
/// owns the whole form.
///
/// A struct rather than a dozen captured variables: `InlineComposer` was longer than the
/// lint allows, and the honest fix is to name the state the submit path needs instead of
/// carving the markup into pieces that each take nine props.
struct Draft {
    text: RwSignal<String>,
    visibility: RwSignal<Visibility>,
    busy: RwSignal<bool>,
    error: RwSignal<Option<String>>,
    notice: RwSignal<Option<String>>,
    custom: RwSignal<String>,
    media: RwSignal<Vec<MediaAttachment>>,
    poll_open: RwSignal<bool>,
    options: RwSignal<Vec<String>>,
    expires_in: RwSignal<u32>,
    multiple: RwSignal<bool>,
    hide_totals: RwSignal<bool>,
    /// Empty means "let the instance guess".
    language: RwSignal<String>,
    alive: Alive,
    which: StoredValue<nodus_sdk::Timeline>,
    set_reload: WriteSignal<u32>,
}

impl Draft {
    fn new(which: StoredValue<nodus_sdk::Timeline>, set_reload: WriteSignal<u32>) -> Self {
        Self {
            text: RwSignal::new(String::new()),
            visibility: RwSignal::new(Visibility::Public),
            busy: RwSignal::new(false),
            error: RwSignal::new(None),
            notice: RwSignal::new(None),
            // `YYYY-MM-DDTHH:mm` as the fields report it, in this browser's zone. Empty
            // means "send it now": scheduling is an exception, not the default path.
            custom: RwSignal::new(String::new()),
            media: RwSignal::new(Vec::new()),
            // A poll is optional and off by default: most posts are not questions.
            poll_open: RwSignal::new(false),
            options: RwSignal::new(vec![String::new(), String::new()]),
            expires_in: RwSignal::new(poll::DURATIONS[0].0),
            multiple: RwSignal::new(false),
            hide_totals: RwSignal::new(false),
            language: RwSignal::new(String::new()),
            alive: Alive::new(),
            which,
            set_reload,
        }
    }

    /// The `Copy` half of the state, so a handler can own it without owning `self`.
    fn snapshot_of_signals(&self) -> Signals {
        Signals {
            text: self.text,
            visibility: self.visibility,
            busy: self.busy,
            error: self.error,
            notice: self.notice,
            custom: self.custom,
            media: self.media,
            poll_open: self.poll_open,
            options: self.options,
            expires_in: self.expires_in,
            multiple: self.multiple,
            hide_totals: self.hide_totals,
            language: self.language,
            alive: self.alive,
            which: self.which,
            set_reload: self.set_reload,
        }
    }
}

/// The `Copy` signals of a draft, so a handler can be `Copy` too.
#[derive(Clone, Copy)]
struct Signals {
    text: RwSignal<String>,
    visibility: RwSignal<Visibility>,
    busy: RwSignal<bool>,
    error: RwSignal<Option<String>>,
    notice: RwSignal<Option<String>>,
    custom: RwSignal<String>,
    media: RwSignal<Vec<MediaAttachment>>,
    poll_open: RwSignal<bool>,
    options: RwSignal<Vec<String>>,
    expires_in: RwSignal<u32>,
    multiple: RwSignal<bool>,
    hide_totals: RwSignal<bool>,
    language: RwSignal<String>,
    alive: Alive,
    which: StoredValue<nodus_sdk::Timeline>,
    set_reload: WriteSignal<u32>,
}

impl Signals {
    fn submit(self) {
        if self.busy.get_untracked() {
            return;
        }
        let body = self.text.get_untracked();
        if body.trim().is_empty() {
            return;
        }
        let mut status = actions::NewStatus::text(body, self.visibility.get_untracked());
        status.media_ids = self
            .media
            .get_untracked()
            .iter()
            .map(|item| item.id.clone())
            .collect();
        status.poll = poll::as_draft(
            self.poll_open.get_untracked(),
            &self.options.get_untracked(),
            self.expires_in.get_untracked(),
            self.multiple.get_untracked(),
            self.hide_totals.get_untracked(),
        );
        status.language = {
            let held = self.language.get_untracked();
            // Empty means the picker is on "instance default", which is the absence of the
            // field — sending `language=` would ask the instance to store a blank code.
            (!held.is_empty()).then_some(held)
        };

        let chosen = self.visibility.get_untracked();
        let custom = self.custom.get_untracked();
        let at = custom_time(&custom);
        if !custom.is_empty() && at.is_none() {
            self.error.set(Some(
                "That is not a date and time this browser can read.".to_owned(),
            ));
            return;
        }

        self.busy.set(true);
        self.error.set(None);
        self.notice.set(None);
        let snapshot = Snapshot {
            status,
            custom,
            text: self.text,
            busy: self.busy,
            error: self.error,
            notice: self.notice,
            media: self.media,
            custom_at: self.custom,
            poll_open: self.poll_open,
            options: self.options,
            alive: self.alive,
            which: self.which,
            set_reload: self.set_reload,
        };
        spawn_local(async move {
            let outcome = match at {
                Some(at) => actions::schedule(&snapshot.status.text, chosen, &at)
                    .await
                    .map(|()| format!("Scheduled: {}", Notice::of(&snapshot.custom))),
                None => actions::publish(&snapshot.status)
                    .await
                    .map(|_| String::new()),
            };
            snapshot.finish(outcome).await;
        });
    }
}

/// What a submit needs after the request is in flight.
struct Snapshot {
    status: actions::NewStatus,
    custom: String,
    text: RwSignal<String>,
    busy: RwSignal<bool>,
    error: RwSignal<Option<String>>,
    notice: RwSignal<Option<String>>,
    media: RwSignal<Vec<MediaAttachment>>,
    custom_at: RwSignal<String>,
    poll_open: RwSignal<bool>,
    options: RwSignal<Vec<String>>,
    alive: Alive,
    which: StoredValue<nodus_sdk::Timeline>,
    set_reload: WriteSignal<u32>,
}

impl Snapshot {
    async fn finish(self, outcome: Result<String, String>) {
        match outcome {
            Ok(scheduled) => {
                // The draft is gone once the instance has it; nothing is kept for a retry,
                // because a retry after a success is a duplicate post.
                if !self.alive.is_alive() {
                    return;
                }
                self.text.set(String::new());
                self.media.set(Vec::new());
                self.custom_at.set(String::new());
                // The poll travelled with the request; leaving the panel open with the same
                // options would post it again on the next click.
                self.poll_open.set(false);
                self.options.set(vec![String::new(), String::new()]);
                // A scheduled post is not in the timeline yet, so refreshing it would fetch
                // the same page and change nothing.
                if scheduled.is_empty() {
                    let _ = timeline::refresh(self.which.get_value(), None).await;
                }
                if self.alive.is_alive() {
                    self.notice
                        .set(Some(scheduled).filter(|line| !line.is_empty()));
                    self.busy.set(false);
                    self.set_reload.update(|generation| *generation += 1);
                }
            }
            Err(message) => {
                if self.alive.is_alive() {
                    self.error.set(Some(message));
                    self.busy.set(false);
                }
            }
        }
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

/// Returns the caret to the box after an emoji is inserted: the picker's own input took it,
/// and a writer who has to click back into the text loses the sentence's place.
fn focus_textarea() {
    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return;
    };
    if let Some(field) = document
        .get_element_by_id("nodus-composer-text")
        .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok())
    {
        let _ = field.focus();
    }
}
