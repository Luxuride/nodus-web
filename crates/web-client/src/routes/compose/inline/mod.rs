//! The composer that sits above the home timeline.
//!
//! Always open, never toggled: writing is the reason the home timeline exists, and a
//! button that expands a form is one decision more than a box that is simply there. It
//! is deliberately the *short* path — text, media, visibility, and a link to the full
//! composer for anything else (scheduling, a content warning, a reply's thread context).

use std::sync::Arc;

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::{MediaAttachment, Visibility};
use nodus_pages::pages::compose::inline::Posted;
use nodus_pages::pages::compose::inline::{
    InlineComposer as InlineCard, InlineComposerProps as InlineCardProps,
};
use wasm_bindgen::JsCast as _;

use crate::routes::compose::attachments::{self, Attachments};
use crate::routes::compose::inline::bar::ComposerBar;
use crate::state::alive::Alive;
use crate::state::bridge::{actions, timeline};

mod bar;
mod drafts_panel;
pub(crate) mod emoji;
mod emoji_toggle;
mod poll;
mod schedule;

use bar::Notice;
use drafts_panel::DraftsPanel;
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

    // `:draft` and the kept-text lifecycle, in one place: this component was already at the
    // line limit, and the resume path is a feature of its own.
    let (draft_token, resume) = draft_controls(&draft);

    // The card, its heading and the three confirmation lines are the view's. What is not:
    // the writer itself, the two popups that sit above it, and the bar under it — the editor
    // owns the contenteditable, the emoji picker owns a document listener, and the bar's
    // panels read the clock. Those arrive as the two slots the card leaves open.
    let body = Arc::new(move || {
        view! {
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
            <DraftsPanel open=draft_token insert=resume/>
            <nodus_ui::Relative>
                <crate::ui::editor::RichEditor
                    text=draft.text.read_only()
                    set_text=draft.text.write_only()
                    emoji=emoji_catalogue
                    placeholder="What's on your mind?"
                    on_submit=Callback::new(move |()| submit.submit())
                    query=draft.emoji_query
                />
                <EmojiHints query=draft.emoji_query.read_only()/>
            </nodus_ui::Relative>
        }
        .into_any()
    }) as ChildrenFn;

    let controls = Arc::new(move || {
        view! {
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
                on_add=Some(Callback::new(super::inline::poll::focus_option))
            />

            <Attachments media=draft.media set_error=draft.error.write_only()/>
        }
        .into_any()
    }) as ChildrenFn;

    // The confirmation reads as the pickers do: the visibility's own word, and the language
    // the instance would have named, not the code that was sent.
    let done = Signal::derive(move || {
        draft.done.get().map(|(visibility, language)| Posted {
            visibility: visibility.as_str().to_owned(),
            language: if language.is_empty() {
                "instance default".to_owned()
            } else {
                crate::ui::language::name_of(&language)
            },
        })
    });

    // The slots are handed over in the props struct: view syntax can pass an affordance but
    // not the absence of one, and the card needs both slots filled.
    InlineCard(InlineCardProps {
        body,
        controls,
        error: Some(draft.error.into()),
        notice: Some(draft.notice.into()),
        done: Some(done),
    })
}

/// True for a body that is only the `:draft` token, which is a command rather than writing.
fn is_command(text: &str) -> bool {
    text.trim().eq_ignore_ascii_case(":draft")
}

/// The `:draft` popup's two halves: when it is open, and what choosing a row does.
///
/// Registers the "keep unsent text" cleanup on the current owner, so it must run inside a
/// component body — which is where it is called from.
fn draft_controls(draft: &Draft) -> (Signal<bool>, Callback<usize>) {
    // A token is open exactly while the draft ends in one, which is how an emoji code is
    // detected too.
    let text = draft.text;
    let open = Signal::derive(move || {
        emoji::pending(&text.get()).is_some_and(|(_, code)| code.eq_ignore_ascii_case("draft"))
    });

    // Leaving with unsent text keeps it: a reload, a route change, a closed tab. A reload
    // tears the module down without running `on_cleanup`, so the document's own unload
    // event is what saves the text there — that is the case a draft exists for.
    let kept_text = draft.text;
    let kept_visibility = draft.visibility;
    let kept_language = draft.language;
    let epoch_at_mount = crate::state::epoch::get();
    let live = StoredValue::new((String::new(), Visibility::Public, String::new()));
    Effect::new(move |_| {
        live.set_value((kept_text.get(), kept_visibility.get(), kept_language.get()));
    });
    let on_unload = wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::Event)>::new(
        move |_event: web_sys::Event| {
            let (text, visibility, language) = live.get_value();
            // The `:draft` token is a command, not writing: keeping it would fill the list with
            // the very word that opens it.
            if !is_command(&text) {
                crate::ui::drafts::keep(&text, visibility, &language);
            }
        },
    );
    if let Some(window) = web_sys::window() {
        let _ = window
            .add_event_listener_with_callback("beforeunload", on_unload.as_ref().unchecked_ref());
    }
    on_unload.forget();
    on_cleanup(move || {
        // A rebuild of this view is not leaving: only a navigation keeps the text.
        if crate::state::epoch::get() == epoch_at_mount {
            return;
        }
        let text = kept_text.get_untracked();
        if !is_command(&text) {
            crate::ui::drafts::keep(
                &text,
                kept_visibility.get_untracked(),
                &kept_language.get_untracked(),
            );
        }
    });

    // Resuming fills the body, takes the pickers with it, and drops the row so it cannot be
    // sent twice.
    let resume_text = draft.text;
    let resume_visibility = draft.visibility;
    let resume_language = draft.language;
    let resume_done = draft.done;
    let resume = Callback::new(move |index: usize| {
        let Some(saved) = crate::ui::drafts::take(index) else {
            return;
        };
        resume_text.set(saved.text);
        resume_visibility.set(crate::ui::drafts::visibility(&saved.visibility));
        resume_language.set(saved.language);
        resume_done.set(None);
        focus_textarea();
    });

    (open, resume)
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
    /// The `:token` the editor is completing, mirrored here so the hint list is daisyUI's.
    emoji_query: RwSignal<Option<crate::ui::editor::EmojiQuery>>,
    /// What the last successful post went out as: visibility and language, shown in the
    /// pickers' own style so the confirmation reads as the same choice, not a second claim.
    done: RwSignal<Option<(Visibility, String)>>,
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
            expires_in: RwSignal::new(nodus_pages::pages::compose::inline::poll::DURATIONS[0].0),
            multiple: RwSignal::new(false),
            hide_totals: RwSignal::new(false),
            language: RwSignal::new(String::new()),
            emoji_query: RwSignal::new(None),
            done: RwSignal::new(None),
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
            done: self.done,
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
    done: RwSignal<Option<(Visibility, String)>>,
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
            done: self.done,
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
    done: RwSignal<Option<(Visibility, String)>>,
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
                // The pickers' answer, echoed back: what actually went out. Only for a real
                // post — a scheduled one names its time in `notice` instead.
                if scheduled.is_empty() {
                    self.done.set(Some((
                        self.status.visibility,
                        self.status.language.clone().unwrap_or_default(),
                    )));
                }
                crate::ui::drafts::clear();
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

/// The hints for the token being typed, anchored at the caret by the editor.
///
/// Keyboard handling lives in the editor (Enter and Tab take the first match, Escape
/// dismisses); the list and its rows are shared, because the inline box and the full
/// composer show the same hints.
#[component]
fn EmojiHints(query: ReadSignal<Option<crate::ui::editor::EmojiQuery>>) -> impl IntoView {
    view! {
        <nodus_pages::pages::compose::inline::hints::EmojiHints
            value=Signal::derive(move || query.get())
            on_accept=Some(Callback::new(move |code: String| {
                let _ = crate::ui::editor::accept_active(&code);
            }))
        />
    }
}
