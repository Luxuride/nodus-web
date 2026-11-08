//! The row under the textarea: media, visibility, when to send, the counter, the button.

use leptos::prelude::*;
use nodus_instance::model::Visibility;

use super::{LIMIT, SchedulePanel, VisibilityPicker};

/// What the green notice says after a post was queued instead of sent.
pub(super) struct Notice;

impl Notice {
    /// `2026-11-09T09:00` reads better with a space than with the `T` the input emits.
    pub(super) fn of(custom: &str) -> String {
        custom.replace('T', " ")
    }
}

/// Every control that describes *how* the post goes out, kept apart from the box that
/// holds the text so neither half outgrows what one screen can read.
///
/// Scheduling is one absolute field rather than a menu of presets: "tomorrow at nine" is
/// the same answer as "2026-11-09T09:00", and a picker that starts at the current minute
/// needs no arithmetic of ours to be correct in the reader's zone. Empty means now.
#[component]
pub(super) fn ComposerBar(
    text: ReadSignal<String>,
    visibility: ReadSignal<Visibility>,
    set_visibility: WriteSignal<Visibility>,
    /// The instant to send at (`YYYY-MM-DDTHH:mm`), or empty to send now.
    at: ReadSignal<String>,
    set_at: WriteSignal<String>,
    busy: ReadSignal<bool>,
    /// Opens the media picker.
    on_add_media: Callback<()>,
    on_submit: Callback<()>,
) -> impl IntoView {
    let count = move || text.get().chars().count();
    // A time in the field means this press queues the post instead of publishing it, and
    // the button says which it will do.
    let scheduling = move || !at.get().trim().is_empty();
    let open = RwSignal::new(false);
    // "Grey like send": a post button that cannot post must look like a button that
    // cannot post, not like a primary action waiting to be pressed.
    let postable = move || !busy.get() && !text.get().trim().is_empty();
    let describe_when = move |value: &str| {
        if value.trim().is_empty() {
            "now".to_owned()
        } else {
            value.replace('T', " ").replacen('-', "/", 2)
        }
    };
    view! {
        <div class="flex flex-wrap items-center gap-2">
            <button
                type="button"
                class="btn btn-outline btn-sm gap-1 border-dashed"
                title="Add images, video or audio"
                disabled=move || busy.get()
                on:click=move |_| on_add_media.run(())
            >
                <span class="text-base leading-none">"+"</span>
                <span>"Media"</span>
            </button>

            <VisibilityPicker visibility=visibility set_visibility=set_visibility/>

            // The count and the action sit at the far end: the left edge answers "how
            // does this go out", the right edge answers "is it ready to".
            <div class="ml-auto flex items-center gap-2">
                <span
                    class="text-xs tabular-nums"
                    class=("opacity-50", move || count() <= LIMIT)
                    class=("text-error", move || count() > LIMIT)
                    title="The instance refuses a post longer than this"
                >
                    {move || format!("{}/{LIMIT}", count())}
                </span>

                // Post and its caret are one control: the caret opens the schedule
                // panel, and the label above says what the pair will do.
                <div class="relative">
                    <div class="join">
                    <button
                        class="btn btn-sm join-item gap-1"
                        class:btn-primary=move || postable()
                        class:bg-base-300=move || !postable()
                        class:opacity-50=move || !postable()
                        disabled=move || !postable()
                        on:click=move |_| on_submit.run(())
                    >
                        {move || match (busy.get(), scheduling()) {
                            (true, _) => "Posting…",
                            (false, true) => "Schedule",
                            (false, false) => "Post",
                        }}
                    </button>
                    <button
                        type="button"
                        class="btn btn-sm join-item px-2"
                        class:btn-primary=move || postable()
                        class:bg-base-300=move || !postable()
                        class:opacity-50=move || !postable()
                        disabled=move || !postable()
                        title=move || format!("Send {}, or pick a time", describe_when(&at.get()))
                        aria-label="Change when this posts"
                        aria-expanded=move || open.get().to_string()
                        on:click=move |_| open.update(|shown| *shown = !*shown)
                    >
                        <span class="text-[0.6rem] leading-none">"▼"</span>
                    </button>
                    </div>
                    // Outside the `join`: the panel is not part of the button group, and
                    // the group's clipping would cut it off.
                    <Show when=move || open.get()>
                        <SchedulePanel at=at set_at=set_at open=open/>
                    </Show>
                </div>
            </div>
        </div>
    }
}
