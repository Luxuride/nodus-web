//! The composer's first paint, without an editor or a transport.
//!
//! The server and a browser before its wasm has booted can still show the card the
//! reader is about to write in. Every control is the same component the client uses;
//! the verbs are inert because there is nothing yet to talk to.

use std::sync::Arc;

use leptos::prelude::*;
use nodus_client_core::model::Visibility;

use super::{ComposerBar, InlineComposer};
use crate::pages::compose::inline::emoji_toggle::EmojiToggleButton;
use crate::{Anchor, LanguagePicker, TimelineTabs};

/// The home composer as the server paints it: same card, same bar, empty editor.
#[component]
pub fn ComposerPreview(#[prop(into)] active: Signal<String>) -> impl IntoView {
    let noop = Callback::new(|()| ());
    let noop_visibility = Callback::new(|_: Visibility| ());
    let noop_language = Callback::new(|_: String| ());
    let language_host = NodeRef::<leptos::html::Div>::new();
    let emoji_host = NodeRef::<leptos::html::Div>::new();

    let body: ChildrenFn = Arc::new(|| {
        view! {
            <div
                id="nodus-composer-body"
                class="min-h-24 w-full cursor-text text-sm leading-relaxed"
                aria-label="Write a post"
            >
                <p class="pointer-events-none opacity-50">"What's on your mind?"</p>
            </div>
        }
        .into_any()
    });

    let language: ChildrenFn = Arc::new(move || {
        view! {
            <LanguagePicker
                selected=Signal::derive(String::new)
                name=Signal::derive(|| "Language".to_owned())
                open=Signal::derive(|| false)
                options=Signal::derive(Vec::new)
                host=language_host
                on_toggle=Some(noop)
                on_pick=Some(noop_language)
            />
        }
        .into_any()
    });

    let emoji_picker: ChildrenFn = Arc::new(move || {
        view! {
            <Anchor host=emoji_host>
                <EmojiToggleButton
                    open=Signal::derive(|| false)
                    busy=Signal::derive(|| false)
                    on_toggle=noop
                />
            </Anchor>
        }
        .into_any()
    });

    let bar_language = Arc::clone(&language);
    let bar_emoji = Arc::clone(&emoji_picker);
    let controls: ChildrenFn = Arc::new(move || {
        let language = Arc::clone(&bar_language);
        let emoji_picker = Arc::clone(&bar_emoji);
        view! {
            <ComposerBar
                text=Signal::derive(String::new)
                visibility=Signal::derive(|| Visibility::Public)
                busy=Signal::derive(|| false)
                scheduling=Signal::derive(|| false)
                when=Signal::derive(String::new)
                limit=500
                schedule_open=Signal::derive(|| false)
                on_visibility=noop_visibility
                on_add_media=noop
                on_add_poll=noop
                on_toggle_schedule=noop
                on_submit=noop
                language=language
                emoji_picker=emoji_picker
            />
        }
        .into_any()
    });

    view! {
        <div class="flex flex-col gap-2">
            <InlineComposer body=body controls=controls/>
            <TimelineTabs active=active/>
        </div>
    }
}
