//! The compose form, shared by writing and editing.
//!
//! One form for both because the wire draft is the same shape: Mastodon's edit is a `PUT`
//! of the same fields a `POST` takes. The only difference is the button's label and which
//! bridge function runs, so the difference is one `if`.

use std::sync::Arc;

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::{MediaAttachment, Status};
use nodus_view::pages::compose::form::{ComposeForm as Form, ComposeFormProps as FormProps};

use self::buttons::ComposerActions;
use self::initial::Initial;
use self::options::ComposerOptions;
use super::attachments::Attachments;
use crate::router::{self, Route};
use crate::state::alive::Alive;
use crate::state::bridge::actions::{self, NewStatus};

// Named `buttons`, not `actions`: the bridge module is already `actions` in this file.
mod buttons;
mod initial;
mod options;

/// Applies the account's posting defaults, without disturbing typing.
///
/// The request is best effort: a composer that cannot reach `/api/v1/preferences`
/// still has to let the reader post, so a failure leaves the built-in defaults in
/// place rather than blocking the form.
fn start_from_preferences(
    set_visibility: WriteSignal<nodus_instance::model::Visibility>,
    set_sensitive: WriteSignal<bool>,
) {
    let alive = crate::state::alive::Alive::new();
    leptos::task::spawn_local(async move {
        if let Ok(preferences) = crate::state::bridge::notices::preferences().await {
            let visibility = preferences.visibility();
            let sensitive = preferences.posting_default_sensitive;
            alive.if_alive(|| {
                set_visibility.set(visibility);
                set_sensitive.set(sensitive);
            });
        }
    });
}

/// The form. `initial` turns it into an editor.
#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "a Leptos component receives its props by value"
)]
pub fn ComposeForm(initial: Option<Status>) -> impl IntoView {
    let start = Initial::of(initial.as_ref());
    let (text, set_text) = signal(start.text);
    let emoji_query = RwSignal::new(None); // no hint list here; the token rule still applies
    let (visibility, set_visibility) = signal(start.visibility);
    let (spoiler, set_spoiler) = signal(start.spoiler);
    let (language, set_language) = signal(String::new()); // empty: let the instance guess
    let (warning_open, set_warning_open) = signal(start.warning_open);
    let (sensitive, set_sensitive) = signal(start.sensitive);
    // Stored rather than captured: the submit handler and the button label both read it,
    // and a captured `String` would make one of them `FnOnce`.
    let editing = StoredValue::new(start.editing);
    let media = RwSignal::new(Vec::<MediaAttachment>::new());
    let (busy, set_busy) = signal(false);
    let (error, set_error) = signal(None::<String>);

    // A new post starts from the account's preferences; an edit starts from the post.
    if editing.get_value().is_none() {
        start_from_preferences(set_visibility, set_sensitive);
    }

    let submit = Callback::new(move |()| {
        let body = text.get_untracked();
        if body.trim().is_empty() {
            set_error.set(Some("A status needs a body.".to_owned()));
            return;
        }
        let status = wire_status(
            body,
            visibility.get_untracked(),
            media.get_untracked(),
            if warning_open.get_untracked() {
                spoiler.get_untracked()
            } else {
                String::new()
            },
            sensitive.get_untracked(),
            language.get_untracked(),
        );
        let editing = editing.get_value();
        let alive = Alive::new();
        set_busy.set(true);
        set_error.set(None);
        spawn_local(async move {
            let answer = match &editing {
                Some(id) => actions::edit(id, &status).await,
                None => actions::publish(&status).await,
            };
            if let Err(message) = answer {
                alive.if_alive(|| {
                    set_error.set(Some(message));
                    set_busy.set(false);
                });
                return;
            }
            // An edit lands on the thread, which shows the new revision; a fresh status has
            // no page of its own, so home is where it appears.
            router::navigate(match &editing {
                Some(id) => Route::Thread {
                    host: crate::state::bridge::session::remembered_host().unwrap_or_default(),
                    id: id.clone(),
                },
                None => Route::Home,
            });
            alive.if_alive(|| set_busy.set(false));
        });
    });

    let label = if editing.get_value().is_some() {
        "Edit this status"
    } else {
        "Say something"
    };
    // The frame — the screen column, the line above the box, the error under it — is the
    // view's; the editor and the three rows under it are this crate's, and the frame takes
    // them as the two slots it leaves open.
    let body = Arc::new(move || {
        view! { <Body text=text set_text=set_text submit=submit query=emoji_query/> }.into_any()
    }) as ChildrenFn;

    let controls = Arc::new(move || {
        view! {
            <ComposerOptions
                visibility=visibility
                set_visibility=set_visibility
                warning_open=warning_open
                set_warning_open=set_warning_open
                sensitive=sensitive
                set_sensitive=set_sensitive
                spoiler=spoiler
                set_spoiler=set_spoiler
                language=language
                set_language=set_language
            />

            <Attachments media=media set_error=set_error/>

            <ComposerActions
                text=text
                busy=busy
                editing=editing
                on_submit=submit
            />
        }
        .into_any()
    }) as ChildrenFn;

    // The slots are handed over in the props struct: view syntax can pass an affordance but
    // not the absence of one, and the frame needs both slots filled.
    Form(FormProps {
        label: label.to_owned(),
        body,
        children: controls,
        error: Some(error.into()),
    })
}

/// The post as the instance will receive it.
///
/// A function rather than an inline block so the component stays inside clippy's line
/// budget: the composer's layout is long enough on its own, and this part is pure data.
fn wire_status(
    text: String,
    visibility: nodus_instance::model::Visibility,
    media: Vec<MediaAttachment>,
    spoiler_text: String,
    sensitive: bool,
    language: String,
) -> NewStatus {
    let mut status = NewStatus::text(text, visibility);
    status.media_ids = media.into_iter().map(|attachment| attachment.id).collect();
    status.spoiler_text = spoiler_text;
    status.sensitive = sensitive;
    status.language = (!language.is_empty()).then_some(language);
    status
}

/// The body field, apart from the form's state so `ComposeForm` stays readable.
#[component]
fn Body(
    text: ReadSignal<String>,
    set_text: WriteSignal<String>,
    submit: Callback<()>,
    query: RwSignal<Option<crate::ui::editor::EmojiQuery>>,
) -> impl IntoView {
    view! {
        <nodus_view::FieldBox>
            <crate::ui::editor::RichEditor
                text=text
                set_text=set_text
                emoji=super::inline::emoji::catalogue()
                on_submit=submit
                query=query
            />
        </nodus_view::FieldBox>
    }
}
