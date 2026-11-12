//! The bar's shell.
//!
//! Every control that describes *how* the post goes out, kept apart from the box that
//! holds the text so neither half outgrows what one screen can read. The row itself — the
//! media and poll buttons, the visibility select, the counter, the post button and its
//! caret — is the view's; the three pieces that are not markup arrive as slots: the
//! language list, the emoji picker (it owns a document listener), and the schedule panel
//! (it reads the reader's clock).
//!
//! Scheduling is one absolute field rather than a menu of presets: "tomorrow at nine" is
//! the same answer as "2026-11-09T09:00", and a picker that starts at the current minute
//! needs no arithmetic of ours to be correct in the reader's zone. Empty means now.

use std::sync::Arc;

use leptos::prelude::*;
use nodus_client_core::model::Visibility;
use nodus_pages::pages::compose::inline::bar::{ComposerBar as Bar, ComposerBarProps as BarProps};

use super::SchedulePanel;

/// What the green notice says after a post was queued instead of sent.
pub(super) struct Notice;

impl Notice {
    /// `2026-11-09T09:00` reads better with a space than with the `T` the input emits.
    pub(super) fn of(custom: &str) -> String {
        custom.replace('T', " ")
    }
}

/// The bar under the textarea.
#[component]
pub(super) fn ComposerBar(
    text: ReadSignal<String>,
    visibility: ReadSignal<Visibility>,
    set_visibility: WriteSignal<Visibility>,
    /// The instant to send at (`YYYY-MM-DDTHH:mm`), or empty to send now.
    at: ReadSignal<String>,
    set_at: WriteSignal<String>,
    busy: ReadSignal<bool>,
    /// The instance's character limit.
    limit: Signal<usize>,
    /// Opens the visual thread editor.
    on_thread: Option<Callback<()>>,
    /// Opens the media picker.
    on_add_media: Callback<()>,
    /// Whether the emoji picker is showing.
    emoji_open: ReadSignal<bool>,
    /// Toggles the emoji picker. It hangs off the emoticon button in the right-hand group,
    /// because it belongs to the body rather than to the "make this a poll/media" row.
    set_emoji_open: WriteSignal<bool>,
    /// Turns the post into a poll.
    on_add_poll: Callback<()>,
    /// The chosen language, empty for the instance's guess.
    language: ReadSignal<String>,
    set_language: WriteSignal<String>,
    on_submit: Callback<()>,
) -> impl IntoView {
    // A time in the field means this press queues the post instead of publishing it, and
    // the button says which it will do.
    let scheduling = Signal::derive(move || !at.get().trim().is_empty());
    // The caret shows the chosen instant in the same shape the fields use.
    let when = Signal::derive(move || at.get());
    let open = RwSignal::new(false);

    let picker = Arc::new(move || {
        view! { <crate::ui::language::LanguagePicker language=language set_language=set_language/> }
            .into_any()
    }) as ChildrenFn;
    let emoji_picker = Arc::new(move || {
        view! { <super::emoji_toggle::EmojiToggle open=emoji_open set_open=set_emoji_open busy=busy/> }
            .into_any()
    }) as ChildrenFn;
    let schedule_panel =
        Arc::new(move || view! { <SchedulePanel at=at set_at=set_at open=open/> }.into_any())
            as ChildrenFn;

    // The props struct is filled directly because view syntax can hand a component a slot but
    // not the absence of one.
    Bar(BarProps {
        text: text.into(),
        visibility: visibility.into(),
        busy: busy.into(),
        scheduling,
        when,
        limit,
        schedule_open: open.into(),
        on_visibility: Some(Callback::new(move |chosen: Visibility| {
            set_visibility.set(chosen)
        })),
        on_add_media: Some(on_add_media),
        on_add_poll: Some(on_add_poll),
        on_toggle_schedule: Some(Callback::new(move |()| {
            open.update(|shown| *shown = !*shown)
        })),
        on_thread,
        on_submit: Some(on_submit),
        language: Some(picker),
        emoji_picker: Some(emoji_picker),
        schedule_panel: Some(schedule_panel),
    })
}
