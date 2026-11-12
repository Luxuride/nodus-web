//! The compose form, shared by writing and editing.
//!
//! One form for both because the wire draft is the same shape: Mastodon's edit is a `PUT`
//! of the same fields a `POST` takes. The only difference is the button's label and which
//! bridge function runs, so the difference is one `if`.

use std::sync::Arc;

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_client_core::model::{MediaAttachment, Status};
use nodus_pages::pages::compose::form::{ComposeForm as Form, ComposeFormProps as FormProps};

use self::buttons::ComposerActions;
use self::initial::Initial;
use self::options::ComposerOptions;
use super::attachments::Attachments;
use crate::router::{self, Route};
use crate::state::alive::Alive;
use crate::state::bridge::actions::{self, NewStatus};
use crate::state::bridge::limits;
use nodus_pages::pages::compose::thread::count;

// Named `buttons`, not `actions`: the bridge module is already `actions` in this file.
mod buttons;
mod initial;
mod options;
mod thread;

/// Applies the account's posting defaults, without disturbing typing.
///
/// The request is best effort: a composer that cannot reach `/api/v1/preferences`
/// still has to let the reader post, so a failure leaves the built-in defaults in
/// place rather than blocking the form.
fn start_from_preferences(
    set_visibility: WriteSignal<nodus_client_core::model::Visibility>,
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
    let handed = super::handoff::take();
    let from_handoff = handed.is_some();
    let start = Initial::of(initial.as_ref());
    let (handed_text, reply_to, handed_media) = handed.map_or_else(
        || (start.text, None, Vec::new()),
        |handed| (handed.text, handed.reply_to, handed.media),
    );
    let (text, set_text) = signal(handed_text);
    let thread_mode = RwSignal::new(from_handoff);
    let limit = RwSignal::new(500usize);
    {
        let alive = Alive::new();
        spawn_local(async move {
            let value = limits::max_characters().await;
            alive.if_alive(|| limit.set(value));
        });
    }
    let emoji_query = RwSignal::new(None); // no hint list here; the token rule still applies
    let (visibility, set_visibility) = signal(start.visibility);
    let (spoiler, set_spoiler) = signal(start.spoiler);
    let (language, set_language) = signal(String::new()); // empty: let the instance guess
    let (warning_open, set_warning_open) = signal(start.warning_open);
    let (sensitive, set_sensitive) = signal(start.sensitive);
    // Stored rather than captured: the submit handler and the button label both read it,
    // and a captured `String` would make one of them `FnOnce`.
    let editing = StoredValue::new(start.editing);
    let media = RwSignal::new(handed_media);
    let pages = RwSignal::new(Vec::<thread::Page>::new());
    let (busy, set_busy) = signal(false);
    let (error, set_error) = signal(None::<String>);

    // A new post starts from the account's preferences; an edit starts from the post.
    if editing.get_value().is_none() {
        start_from_preferences(set_visibility, set_sensitive);
    }

    let submit = Callback::new(move |()| {
        let body = text.get_untracked();
        let thread = thread_mode.get_untracked();
        let layouts = if thread {
            thread::texts(&pages.get_untracked())
        } else {
            vec![body.clone()]
        };
        if layouts.iter().all(|text| text.trim().is_empty()) {
            set_error.set(Some("A status needs a body.".to_owned()));
            return;
        }
        let limit = limit.get_untracked();
        if layouts
            .iter()
            .any(|text| !text.trim().is_empty() && count(text) > limit)
        {
            set_error.set(Some(format!(
                "This server allows {limit} characters per post. Split the page that is too long first."
            )));
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
            reply_to.clone(),
        );
        let editing = editing.get_value();
        let parts = if editing.is_some() || !thread {
            vec![status]
        } else {
            super::thread::statuses_from(layouts, status, limit)
        };
        let alive = Alive::new();
        set_busy.set(true);
        set_error.set(None);
        spawn_local(async move {
            let answer = match &editing {
                Some(id) => actions::edit(id, &parts[0])
                    .await
                    .map(|status| vec![status]),
                None => {
                    let outcome = actions::publish_thread(&parts).await;
                    if outcome.statuses.is_empty() {
                        Err(outcome.error.unwrap_or_default())
                    } else {
                        Ok(outcome.statuses)
                    }
                }
            };
            let posted = match answer {
                Ok(posted) => posted,
                Err(message) => {
                    alive.if_alive(|| {
                        set_error.set(Some(message));
                        set_busy.set(false);
                    });
                    return;
                }
            };
            // An edit lands on the thread, which shows the new revision; a thread lands on
            // its first post; a single fresh status has no page of its own, so home.
            router::navigate(match &editing {
                Some(id) => Route::Thread {
                    host: crate::state::bridge::session::remembered_host().unwrap_or_default(),
                    id: id.clone(),
                },
                None => match posted.first() {
                    Some(status) if posted.len() > 1 => Route::Thread {
                        host: crate::state::bridge::session::remembered_host().unwrap_or_default(),
                        id: status.id.clone(),
                    },
                    _ => Route::Home,
                },
            });
            alive.if_alive(|| set_busy.set(false));
        });
    });

    let label = if editing.get_value().is_some() {
        "Edit this post"
    } else {
        "New post"
    };
    // The frame — the screen column, the line above the box, the error under it — is the
    // view's; the editor and the three rows under it are this crate's, and the frame takes
    // them as the two slots it leaves open.
    let body = Arc::new(move || {
        view! {
            <Show when=move || !thread_mode.get()>
                <Body text=text set_text=set_text submit=submit query=emoji_query/>
            </Show>
            {move || {
                (!editing.get_value().is_some() && thread_mode.get()).then(|| view! {
                    <thread::ThreadPages
                        text=text
                        pages=pages
                        set_error=set_error
                        limit=limit.into()
                    />
                })
            }}
        }
        .into_any()
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
                limit=limit.into()
                extra=Signal::derive(move || pages.with(Vec::len).saturating_sub(1))
                thread_mode=thread_mode.into()
                invalid=Signal::derive(move || {
                    let max = limit.get();
                    (!thread_mode.get() && count(&text.get()) > max)
                        || pages.with(|pages| {
                            pages.iter().any(|page| count(&page.text.get()) > max)
                        })
                })
                on_thread=Callback::new(move |()| thread_mode.set(true))
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
    visibility: nodus_client_core::model::Visibility,
    media: Vec<MediaAttachment>,
    spoiler_text: String,
    sensitive: bool,
    language: String,
    reply_to: Option<String>,
) -> NewStatus {
    let mut status = NewStatus::text(text, visibility);
    status.reply_to = reply_to;
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
    query: RwSignal<Option<nodus_editor::EmojiQuery>>,
) -> impl IntoView {
    view! {
        <nodus_ui::FieldBox>
            <nodus_editor::RichEditor
                text=text
                set_text=set_text
                emoji=super::inline::emoji::catalogue()
                on_submit=submit
                query=query
            />
        </nodus_ui::FieldBox>
    }
}
