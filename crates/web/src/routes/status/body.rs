//! The part of a status that is content: text, content warning, media, poll.
//!
//! Split from the card so neither file is about two things: this one renders what the
//! author wrote, the card renders who wrote it and what a reader can do about it.

use leptos::prelude::*;
use nodus_instance::model::{MediaAttachment, MediaKind, Status};

use super::poll::PollView;
use super::sensitive;
use crate::routes::feed::posts::plain_text;

/// Content, media and poll of one status.
#[component]
pub fn StatusBody(shown: ReadSignal<Status>, set_shown: WriteSignal<Status>) -> impl IntoView {
    // Per status, not global: a reader who opens one post's warning has not consented to
    // the next one, and the flag is a claim about this attachment set only.
    let (revealed, set_revealed) = signal(sensitive::reveals_automatically());
    let hidden = move || shown.get().sensitive && !revealed.get();

    view! {
        <Show when=move || !shown.get().spoiler_text.trim().is_empty()>
            <div class="rounded-box bg-base-200 p-3">
                <p class="text-sm font-medium">{move || shown.get().spoiler_text.clone()}</p>
                // Opening the warning is consent for the whole post: text, media and poll
                // arrive together, and asking twice would make one decision cost two clicks.
                <details
                    class="mt-2"
                    on:toggle=move |event| {
                        if event_target::<web_sys::HtmlDetailsElement>(&event).open() {
                            set_revealed.set(true);
                        }
                    }
                >
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

        <Show when=move || hidden()>
            // Blurred *and* labelled: a blur alone leaves the reader guessing whether the
            // hidden thing is a photo, a chart or a joke, which is worse than a warning.
            <div class="relative overflow-hidden rounded-box">
                <MediaGrid items=Signal::derive(move || shown.get().media_attachments) hidden=true/>
                // Only a label and a click target: the sharpest thing on screen should not
                // be the overlay, and a veil over a blur reads as a bug rather than a gate.
                <button
                    type="button"
                    class="absolute inset-0 flex cursor-pointer items-center justify-center bg-transparent"
                    on:click=move |_| set_revealed.set(true)
                >
                    <span class="rounded-box bg-base-300/90 px-3 py-1.5 text-center shadow-sm transition-transform hover:scale-105">
                        <span class="block text-sm font-medium">"Sensitive content"</span>
                        <span class="block text-xs opacity-70">{move || count_label(&shown.get())}</span>
                    </span>
                </button>
            </div>
        </Show>

        <Show when=move || !shown.get().media_attachments.is_empty() && !hidden()>
            <MediaGrid items=Signal::derive(move || shown.get().media_attachments) hidden=false/>
        </Show>

        <Show when=move || shown.get().poll.is_some()>
            <PollView shown=shown set_shown=set_shown/>
        </Show>
    }
}

/// "Show 3 attachments", singular for one.
fn count_label(status: &Status) -> String {
    let count = status.media_attachments.len();
    if count == 1 {
        "Show 1 attachment".to_owned()
    } else {
        format!("Show {count} attachments")
    }
}

/// The attachment grid, which is also the gallery: clicking one picture opens the viewer
/// on that position, so a multi-image post can be read in order.
#[component]
fn MediaGrid(items: Signal<Vec<MediaAttachment>>, hidden: bool) -> impl IntoView {
    // Collected outside the macro: a turbofish inside `view!` ends the tag early.
    let rows = move || items.get().into_iter().enumerate().collect::<Vec<_>>();
    // The grid must be exactly as wide as the pictures it holds, or a centred label lands
    // in the gap beside a lone image.
    let columns = move || {
        if items.get().len() == 1 {
            "grid grid-cols-1 gap-2"
        } else {
            "grid grid-cols-2 gap-2"
        }
    };

    view! {
        <ul class=columns aria-hidden=hidden.to_string()>
            <For each=rows key=|(_, media)| media.id.clone() let:entry>
                <MediaTile items=items media=entry.1 interactive=!hidden blurred=hidden/>
            </For>
        </ul>
    }
}

/// One attachment.
///
/// The strings live in signals rather than locals because `Show`'s children closure must
/// be `Fn`: a `String` captured by value makes it `FnOnce`, and a signal is `Copy`.
#[component]
fn MediaTile(
    items: Signal<Vec<MediaAttachment>>,
    media: MediaAttachment,
    interactive: bool,
    blurred: bool,
) -> impl IntoView {
    let is_image = media.kind == MediaKind::Image;
    let kind = media.kind;
    let id = StoredValue::new(media.id.clone());
    let (url, _) = signal(media.url.clone().unwrap_or_default());
    let (preview, _) = signal(media.preview_url.clone().unwrap_or_default());
    let (description, _) = signal(media.description.clone().unwrap_or_default());
    // The instance reports the original's size, and the preview keeps the same ratio, so
    // declaring it reserves the row before a byte arrives. Without it every image load
    // pushed the page down by its own height, which is what "the website jumps" means.
    let (width, _) = signal(media.meta_width);
    let (height, _) = signal(media.meta_height);

    let open_viewer = move |_| {
        let all = items.get_untracked();
        let pictures: Vec<crate::media::Picture> = all
            .iter()
            .filter(|media| media.kind == MediaKind::Image)
            .map(|media| crate::media::Picture {
                url: media.url.clone().unwrap_or_default(),
                alt: media.description.clone().unwrap_or_default(),
            })
            .collect();
        let position = all
            .iter()
            .filter(|media| media.kind == MediaKind::Image)
            .position(|media| media.id == id.get_value())
            .unwrap_or(0);
        crate::media::expand(pictures, position);
    };

    // Inline rather than utility classes: the gate must survive a stylesheet that predates
    // it, and these are two declarations. The scale is not decoration — a blur samples
    // outside the element as transparent, so an unscaled blur eats its own edges and reads
    // as a hole rather than as hidden content.
    let veil = if blurred {
        "filter: blur(18px); transform: scale(1.08);"
    } else {
        ""
    };

    view! {
        <li class="overflow-hidden rounded-box">
            <Show when=move || is_image && interactive>
                // A click expands in place, on the picture the reader pointed at.
                <button type="button" class="w-full cursor-zoom-in" on:click=open_viewer>
                    <img
                        class="h-auto w-full rounded-box bg-base-300 object-cover"
                        style=veil
                        src=move || preview.get()
                        alt=move || description.get()
                        title=move || description.get()
                        width=move || width.get().map(|width| width.to_string()).unwrap_or_default()
                        height=move || height.get().map(|height| height.to_string()).unwrap_or_default()
                        loading="lazy"
                        decoding="async"
                        referrerpolicy="no-referrer"
                    />
                </button>
            </Show>
            <Show when=move || is_image && !interactive>
                <img
                    class="h-auto w-full rounded-box bg-base-300 object-cover"
                    style=veil
                    src=move || preview.get()
                    alt=move || description.get()
                    width=move || width.get().map(|width| width.to_string()).unwrap_or_default()
                    height=move || height.get().map(|height| height.to_string()).unwrap_or_default()
                    loading="lazy"
                    decoding="async"
                    referrerpolicy="no-referrer"
                />
            </Show>
            <Show when=move || !is_image && interactive>
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
