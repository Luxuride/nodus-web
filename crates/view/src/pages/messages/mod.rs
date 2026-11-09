//! The messages screen: direct conversations, newest first.
//!
//! A conversation is a group of accounts and the last status they sent, which is why the
//! screen shows the last message rather than a message list: opening one is a thread. The row
//! itself is shared with the server paint ([`ConversationRow`]); what this page adds is the
//! column around it — the heading, the container, the empty line, the paging control and the
//! error line.
//!
//! Two pieces of the row are the caller's. The newest status is a slot (the server has a
//! rendered card, the client a one-line preview), and the mute toggle sits in the row's
//! trailing slot because the mute state is not part of the conversation payload — the screen
//! that remembers it is the one that can undo it. Without the toggle's callback the button
//! keeps its label and its box and is disabled.

use std::sync::Arc;

use leptos::prelude::*;
use nodus_instance::model::Conversation;

use crate::chrome::Skeleton;
use crate::chrome::more_button::{MoreButton, MoreButtonProps};
use crate::lists::conversation_row::{ConversationRow, ConversationRowProps};
use crate::{PageHeader, Screen};

/// The conversations the account is part of.
#[component]
pub fn MessagesPage(
    /// The conversations, newest first, as the caller last read them.
    conversations: Signal<Vec<Conversation>>,
    /// The ids the viewer muted, which the conversation payload does not carry.
    muted: Signal<Vec<String>>,
    /// Whether a read is already on its way.
    loading: Signal<bool>,
    /// The last read's refusal, when it had one.
    error: Signal<Option<String>>,
    /// The cursor of the page after this one, when the instance has more.
    cursor: Signal<Option<String>>,
    /// Formats a timestamp as an age. Absent when the caller has no clock to lend.
    #[prop(optional)]
    age_of: Option<Callback<String, String>>,
    /// Opens the conversation, which on the client also marks it read.
    #[prop(optional)]
    on_open: Option<Callback<String>>,
    /// Marks the conversation read without opening it.
    #[prop(optional)]
    on_read: Option<Callback<String>>,
    /// Mutes or unmutes one conversation: the id, and whether to mute.
    #[prop(optional)]
    on_mute: Option<Callback<(String, bool)>>,
    /// Asks for the page after the given cursor.
    #[prop(optional)]
    on_more: Option<Callback<String>>,
) -> impl IntoView {
    // The row's age is the caller's clock, asked once per row so the page reads no time of
    // its own.
    let age_of_created = move |created_at: &str| {
        age_of.map_or_else(String::new, |age_of| age_of.run(created_at.to_owned()))
    };

    view! {
        <Screen>
            <PageHeader
                title="Messages"
                description="Direct conversations with other accounts. Opening one marks it read."
            />

            // Only before the first answer: a held list renders immediately, and an empty
            // held list is a fact, not a wait.
            <Show when=move || loading.get() && conversations.get().is_empty()>
                <Skeleton count=4/>
            </Show>

            {move || {
                error
                    .get()
                    .map(|message| {
                        view! {
                            <div class="alert alert-error py-2 text-sm"><span>{message}</span></div>
                        }
                    })
            }}

            <ul class="flex flex-col divide-y divide-base-300 rounded-box bg-base-100">
                <For
                    each=move || conversations.get()
                    key=|conversation| conversation.id.clone()
                    let:conversation
                >
                    {
                        let is_muted = muted.get().contains(&conversation.id);
                        let age = conversation
                            .last_status
                            .as_ref()
                            .map_or_else(String::new, |status| age_of_created(&status.created_at));
                        let preview = conversation
                            .last_status
                            .as_ref()
                            .map(|status| status.content.clone())
                            .unwrap_or_default();
                        // The row is handed its whole set of props: view syntax can pass an
                        // affordance but not the absence of one, and this page only knows what it
                        // was given.
                        ConversationRow(ConversationRowProps {
                            muted: is_muted,
                            age,
                            after: Some(mute_slot(conversation.id.clone(), is_muted, on_mute)),
                            status: Some(preview_slot(preview)),
                            on_open,
                            on_read,
                            conversation,
                        })
                    }
                </For>
            </ul>

            // An empty list is a state, not an error: it is said once, after the first read
            // has finished, and never while the list is still on its way.
            <Show when=move || {
                !loading.get() && conversations.get().is_empty() && error.get().is_none()
            }>
                <p class="text-sm opacity-60">"No conversations yet."</p>
            </Show>

            {move || {
                let next = cursor.get();
                MoreButton(MoreButtonProps {
                    cursor: next,
                    loading: loading.get(),
                    on_more,
                })
            }}
        </Screen>
    }
}

/// The newest status, in the row's middle box: one truncated line, as the client drew it
/// before the row was shared.
fn preview_slot(text: String) -> ChildrenFn {
    Arc::new(move || view! { <p class="truncate text-xs opacity-70">{text.clone()}</p> }.into_any())
        as ChildrenFn
}

/// One conversation's Mute/Unmute toggle, disabled when the caller has no verb for it.
///
/// The verb is the opposite of the state the row was drawn with, so a press always asks for
/// the change the label promises.
fn mute_slot(id: String, muted: bool, on_mute: Option<Callback<(String, bool)>>) -> ChildrenFn {
    Arc::new(move || {
        let id = id.clone();
        view! {
            <button
                class="btn btn-ghost btn-xs"
                disabled=on_mute.is_none()
                on:click=move |_| {
                    if let Some(on_mute) = on_mute {
                        on_mute.run((id.clone(), !muted));
                    }
                }
            >
                {if muted { "Unmute" } else { "Mute" }}
            </button>
        }
        .into_any()
    }) as ChildrenFn
}

#[cfg(test)]
mod rendering {
    use leptos::prelude::{Owner, Signal};
    use leptos::tachys::view::RenderHtml as _;

    use super::MessagesPage;
    use super::MessagesPageProps;

    /// The server paints pages outside a component tree, so every page must survive being
    /// rendered with nothing but an owner. This is the regression guard for
    /// `panicked at leptos-0.8.23/src/for_loop.rs:138: no reactive owner`.
    #[test]
    fn an_empty_page_paints_without_a_component_tree() {
        let html = Owner::new().with(|| {
            MessagesPage(MessagesPageProps {
                conversations: Signal::derive(Vec::new),
                muted: Signal::derive(Vec::new),
                loading: Signal::derive(|| false),
                error: Signal::derive(|| None),
                cursor: Signal::derive(|| None),
                age_of: None,
                on_open: None,
                on_read: None,
                on_mute: None,
                on_more: None,
            })
            .to_html()
        });

        assert!(
            html.contains("Messages"),
            "the page header is painted: {html}"
        );
    }

    /// A row is where the loop actually walks; an empty list never reaches `keyed`.
    #[test]
    fn a_page_with_a_row_paints_without_a_component_tree() {
        let json = r#"{
            "id": "1", "unread": true,
            "accounts": [{"id": "2", "username": "ash", "acct": "ash", "display_name": "Ash",
                          "url": "https://stub/@ash", "note": "", "avatar": "", "header": "",
                          "followers_count": 0, "following_count": 0, "statuses_count": 0,
                          "created_at": "2026-01-01T00:00:00Z", "emojis": []}],
            "last_status": {"id": "9", "content": "<p>hi</p>", "created_at": "2026-01-01T00:00:00Z",
                            "account": {"id": "2", "username": "ash", "acct": "ash", "display_name": "Ash",
                                        "url": "https://stub/@ash", "note": "", "avatar": "", "header": "",
                                        "followers_count": 0, "following_count": 0, "statuses_count": 0,
                                        "created_at": "2026-01-01T00:00:00Z", "emojis": []},
                            "reblog": null, "media_attachments": [], "mentions": [], "tags": [],
                            "favourites_count": 0, "reblogs_count": 0, "replies_count": 0,
                            "favourited": false, "reblogged": false, "bookmarked": false,
                            "sensitive": false, "spoiler_text": "", "visibility": "public",
                            "edited_at": null, "pinned": null, "poll": null, "emojis": []}
        }"#;
        let conversation: nodus_instance::model::Conversation =
            serde_json::from_str(json).expect("the fixture is a conversation");

        let html = Owner::new().with(|| {
            MessagesPage(MessagesPageProps {
                conversations: Signal::derive(move || vec![conversation.clone()]),
                muted: Signal::derive(Vec::new),
                loading: Signal::derive(|| false),
                error: Signal::derive(|| None),
                cursor: Signal::derive(|| None),
                age_of: None,
                on_open: None,
                on_read: None,
                on_mute: None,
                on_more: None,
            })
            .to_html()
        });

        assert!(html.contains("hi"), "the row is painted: {html}");
    }
}
