//! The part of a status that is content: text, content warning, media, poll.
//!
//! Split from the card so neither file is about two things: this one renders what the
//! author wrote, the card renders who wrote it and what a reader can do about it.

use leptos::prelude::*;
use nodus_instance::model::Status;

use super::chips::StatusChips;
use nodus_view::media_grid::Picture;
use super::poll::PollView;
use super::sensitive;

/// Content, media and poll of one status.
#[component]
pub fn StatusBody(shown: ReadSignal<Status>, set_shown: WriteSignal<Status>) -> impl IntoView {
    // Per status, not global: a reader who opens one post's warning has not consented to
    // the next one, and the flag is a claim about this attachment set only.
    let (revealed, set_revealed) = signal(sensitive::reveals_automatically());
    let hidden = move || shown.get().sensitive && !revealed.get();
    // The gallery position is the driver's business, so the shared grid only reports the
    // press; opening the viewer stays here.
    let on_open = Callback::new(move |(pictures, position): (Vec<Picture>, usize)| {
        crate::media::expand(pictures, position);
    });

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
                        <span>
                            {move || view! {
                                <div
                                    class="break-words [&_a]:link [&_a]:link-primary [&_a]:underline-offset-2 [&_p+p]:mt-2"
                                    inner_html=shown.get().content
                                />
                            }}
                        </span>
                    </p>
                </details>
            </div>
        </Show>
        <Show when=move || shown.get().spoiler_text.trim().is_empty()>
            <p class="whitespace-pre-line text-sm leading-relaxed">
                <span>
                            {move || view! {
                                <div
                                    class="break-words [&_a]:link [&_a]:link-primary [&_a]:underline-offset-2 [&_p+p]:mt-2"
                                    inner_html=shown.get().content
                                />
                            }}
                        </span>
            </p>
        </Show>

        <StatusChips shown=shown/>

        <Show when=move || hidden()>
            // Blurred *and* labelled: a blur alone leaves the reader guessing whether the
            // hidden thing is a photo, a chart or a joke, which is worse than a warning.
            <div class="relative overflow-hidden rounded-box">
                <nodus_view::media_grid::MediaGrid items=Signal::derive(move || shown.get().media_attachments) hidden=true on_open=on_open/>
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
            <nodus_view::media_grid::MediaGrid items=Signal::derive(move || shown.get().media_attachments) hidden=false on_open=on_open/>
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
