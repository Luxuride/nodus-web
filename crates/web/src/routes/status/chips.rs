//! The tags, mentions and permalink of a status.
//!
//! A post's body is text, and text loses the two things a reader wants to click: who was
//! addressed and what it was about. The instance already lists both, so this renders them
//! as chips rather than trying to parse them back out of the HTML.

use std::fmt::Write as _;

use leptos::prelude::*;
use nodus_instance::model::Status;

use crate::ui::icons::{Icon, Icon as Glyph};

/// Hashtags, mentions and the permalink, in that order.
#[component]
pub fn StatusChips(shown: ReadSignal<Status>) -> impl IntoView {
    view! {
        <div class="flex flex-wrap items-center gap-1.5">
            <For
                each=move || shown.get().tags
                key=|tag| format!("tag-{}", tag.name)
                let:tag
            >
                {
                    let href = format!("/search?q={}", encode(&format!("#{}", tag.name)));
                    view! {
                        <a class="badge badge-outline badge-sm gap-1" href=href title="Search this hashtag">
                            <Icon icon=Glyph::Tag class="size-3"/>
                            {format!("#{}", tag.name)}
                        </a>
                    }
                }
            </For>
            <For
                each=move || shown.get().mentions
                key=|mention| format!("mention-{}", mention.acct)
                let:mention
            >
                {
                    let href = format!("/search?q={}", encode(&format!("@{}", mention.acct)));
                    view! {
                        <a class="badge badge-ghost badge-sm gap-1" href=href title="Search this account">
                            <Icon icon=Glyph::At class="size-3"/>
                            {format!("@{}", mention.acct)}
                        </a>
                    }
                }
            </For>
            <Show when=move || shown.get().url.is_some()>
                {
                    let url = move || shown.get().url.clone().unwrap_or_default();
                    view! {
                        <a
                            class="ml-auto flex items-center gap-1 text-xs opacity-60 hover:opacity-100"
                            href=url
                            target="_blank"
                            rel="nofollow noopener noreferrer"
                            title="Open this post on its own instance"
                        >
                            <Icon icon=Icon::Link class="size-3"/>
                            "Original"
                        </a>
                    }
                }
            </Show>
        </div>
    }
}

/// Percent-encodes what a query string cannot carry, so `#rust` is not read as a fragment.
fn encode(text: &str) -> String {
    let mut encoded = String::with_capacity(text.len() + 4);
    for byte in text.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(char::from(byte));
            }
            _ => {
                let _ = write!(encoded, "%{byte:02X}");
            }
        }
    }
    encoded
}
