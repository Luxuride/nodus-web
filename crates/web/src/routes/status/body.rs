//! The part of a status that is content: text, content warning, media, poll.
//!
//! Split from the card so neither file is about two things: this one renders what the
//! author wrote, the card renders who wrote it and what a reader can do about it.

use leptos::prelude::*;
use nodus_instance::model::{MediaAttachment, MediaKind, Status};

use super::poll::PollView;
use crate::routes::feed::posts::plain_text;

/// Content, media and poll of one status.
#[component]
pub fn StatusBody(shown: ReadSignal<Status>, set_shown: WriteSignal<Status>) -> impl IntoView {
    view! {
        <Show when=move || !shown.get().spoiler_text.trim().is_empty()>
            <div class="rounded-box bg-base-200 p-3">
                <p class="text-sm font-medium">{move || shown.get().spoiler_text.clone()}</p>
                <details class="mt-2">
                    <summary class="cursor-pointer text-xs opacity-60">"Show content"</summary>
                    <p class="mt-2 whitespace-pre-line text-sm leading-relaxed">
                        {move || plain_text(&shown.get().content)}
                    </p>
                </details>
            </div>
        </Show>
        <Show when=move || shown.get().spoiler_text.trim().is_empty()>
            <p class="whitespace-pre-line text-sm leading-relaxed">
                {move || plain_text(&shown.get().content)}
            </p>
        </Show>

        <Show when=move || !shown.get().media_attachments.is_empty()>
            <ul class="grid grid-cols-2 gap-2">
                <For
                    each=move || shown.get().media_attachments
                    key=|media| media.id.clone()
                    let:media
                >
                    <MediaTile media=media/>
                </For>
            </ul>
        </Show>

        <Show when=move || shown.get().poll.is_some()>
            <PollView shown=shown set_shown=set_shown/>
        </Show>
    }
}

/// One attachment.
///
/// The strings live in signals rather than locals because `Show`'s children closure must
/// be `Fn`: a `String` captured by value makes it `FnOnce`, and a signal is `Copy`.
#[component]
fn MediaTile(media: MediaAttachment) -> impl IntoView {
    let is_image = media.kind == MediaKind::Image;
    let kind = media.kind;
    let (url, _) = signal(media.url.unwrap_or_default());
    let (preview, _) = signal(media.preview_url.unwrap_or_default());
    let (description, _) = signal(media.description.unwrap_or_default());

    view! {
        <li>
            <Show when=move || is_image>
                <a
                    href=move || url.get()
                    target="_blank"
                    rel="nofollow noopener noreferrer"
                    referrerpolicy="no-referrer"
                >
                    <img
                        class="w-full rounded-box bg-base-300 object-cover"
                        src=move || preview.get()
                        alt=move || description.get()
                        title=move || description.get()
                        loading="lazy"
                        referrerpolicy="no-referrer"
                    />
                </a>
            </Show>
            <Show when=move || !is_image>
                <a
                    class="btn btn-ghost btn-sm w-full"
                    href=move || url.get()
                    target="_blank"
                    rel="nofollow noopener noreferrer"
                    referrerpolicy="no-referrer"
                >
                    {format!("Open {kind:?}")}
                </a>
            </Show>
        </li>
    }
}
