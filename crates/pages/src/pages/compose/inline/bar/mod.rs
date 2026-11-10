//! The row under the textarea: media, visibility, when to send, the counter, the button.
//!
//! Every control here is markup; what a press *does* arrives as a verb. The three controls
//! that are not markup — the language picker's list, the emoji picker's document listener,
//! the schedule panel's clock — arrive as slots.
//!
//! Scheduling is one absolute field rather than a menu of presets: "tomorrow at nine" is
//! the same answer as "2026-11-09T09:00", and a picker that starts at the current minute
//! needs no arithmetic of ours to be correct in the reader's zone. Empty means now.

use leptos::prelude::*;
use nodus_instance::model::Visibility;

use crate::components::icons::{Icon, Icon::Image, Icon::List};

/// The bar and the two buttons it ends with.
#[component]
#[expect(
    clippy::too_many_lines,
    reason = "one row of composer controls: splitting it would hide that it is one row"
)]
pub fn ComposerBar(
    #[prop(into)] text: Signal<String>,
    #[prop(into)] visibility: Signal<Visibility>,
    #[prop(into)] busy: Signal<bool>,
    /// Whether a time is chosen, which makes the button queue instead of publish.
    #[prop(into)]
    scheduling: Signal<bool>,
    /// The chosen instant in the shape the fields show it, for the caret's title.
    #[prop(into)]
    when: Signal<String>,
    /// The length above which the instance refuses the post.
    limit: usize,
    /// Whether the schedule panel is showing.
    #[prop(into)]
    schedule_open: Signal<bool>,
    /// Picks who may see the post.
    #[prop(optional)]
    on_visibility: Option<Callback<Visibility>>,
    /// Opens the media picker.
    #[prop(optional)]
    on_add_media: Option<Callback<()>>,
    /// Turns the post into a poll.
    #[prop(optional)]
    on_add_poll: Option<Callback<()>>,
    /// Toggles the schedule panel.
    #[prop(optional)]
    on_toggle_schedule: Option<Callback<()>>,
    /// Sends, or queues, the post.
    #[prop(optional)]
    on_submit: Option<Callback<()>>,
    /// The language picker, whose list is the instance's.
    #[prop(optional)]
    language: Option<ChildrenFn>,
    /// The emoticon and the picker it opens: the panel owns a document listener.
    #[prop(optional)]
    emoji_picker: Option<ChildrenFn>,
    /// The schedule panel: it reads the reader's clock.
    #[prop(optional)]
    schedule_panel: Option<ChildrenFn>,
) -> impl IntoView {
    let count = move || text.get().chars().count();
    // "Grey like send": a post button that cannot post must look like a button that
    // cannot post, not like a primary action waiting to be pressed.
    let postable = move || !busy.get() && !text.get().trim().is_empty() && on_submit.is_some();
    let describe_when = move |value: &str| {
        if value.trim().is_empty() {
            "now".to_owned()
        } else {
            // The same shape the fields show, so the button and the inputs agree.
            value.replace('T', " ")
        }
    };
    let panel = StoredValue::new(schedule_panel);

    view! {
        <div class="flex flex-wrap items-center gap-2">
            <button
                type="button"
                class="btn btn-outline btn-sm gap-1 border-dashed max-sm:btn-square"
                title="Add images, video or audio"
                aria-label="Add images, video or audio"
                disabled=move || busy.get() || on_add_media.is_none()
                on:click=move |_| {
                    if let Some(on_add_media) = on_add_media {
                        on_add_media.run(());
                    }
                }
            >
                <Icon icon=Image/>
                <span class="max-sm:hidden">"Media"</span>
            </button>

            <button
                type="button"
                class="btn btn-outline btn-sm gap-1 border-dashed max-sm:btn-square"
                title="Ask a question instead"
                aria-label="Ask a question instead"
                disabled=move || busy.get() || on_add_poll.is_none()
                on:click=move |_| {
                    if let Some(on_add_poll) = on_add_poll {
                        on_add_poll.run(());
                    }
                }
            >
                <Icon icon=List/>
                <span class="max-sm:hidden">"Poll"</span>
            </button>

            <VisibilityPicker visibility=visibility on_visibility=on_visibility/>

            // The count and the action sit at the far end: the left edge answers "how
            // does this go out", the right edge answers "is it ready to".
            {language.map(|language| language())}
            <div class="ml-auto flex items-center gap-2">
                {emoji_picker.map(|emoji_picker| emoji_picker())}

                <span
                    class="text-xs tabular-nums"
                    class=("hidden", move || count() == 0)
                    class=("opacity-50", move || count() <= limit)
                    class=("text-error", move || count() > limit)
                    title="The instance refuses a post longer than this"
                >
                    {move || format!("{}/{limit}", count())}
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
                        on:click=move |_| {
                            if let Some(on_submit) = on_submit {
                                on_submit.run(());
                            }
                        }
                    >
                        {move || match (busy.get(), scheduling.get()) {
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
                        title=move || if scheduling.get() { format!("Scheduled for {} — click to change", describe_when(&when.get())) } else { "Send now, or pick a time".to_owned() }
                        aria-label="Change when this posts"
                        aria-expanded=move || schedule_open.get().to_string()
                        on:click=move |_| {
                            if let Some(on_toggle_schedule) = on_toggle_schedule {
                                on_toggle_schedule.run(());
                            }
                        }
                    >
                        <span class="text-[0.6rem] leading-none">"▼"</span>
                    </button>
                    </div>
                    // Outside the `join`: the panel is not part of the button group, and
                    // the group's clipping would cut it off.
                    {move || {
                        if schedule_open.get() {
                            panel.with_value(|panel| {
                                panel.as_ref().map_or_else(|| ().into_any(), |panel| panel())
                            })
                        } else {
                            ().into_any()
                        }
                    }}
                </div>
            </div>
        </div>
    }
}

/// The four visibilities, as a select.
#[component]
fn VisibilityPicker(
    #[prop(into)] visibility: Signal<Visibility>,
    on_visibility: Option<Callback<Visibility>>,
) -> impl IntoView {
    view! {
        <select
            class="select select-bordered select-sm w-fit max-sm:w-24"
            aria-label="Who can see this post"
            disabled=move || on_visibility.is_none()
            on:change=move |event| {
                if let Some(on_visibility) = on_visibility {
                    let value = event_target_value(&event);
                    on_visibility
                        .run(match value.as_str() {
                            "unlisted" => Visibility::Unlisted,
                            "private" => Visibility::Private,
                            "direct" => Visibility::Direct,
                            _ => Visibility::Public,
                        });
                }
            }
        >
            <For
                each=|| [
                    (Visibility::Public, "Public"),
                    (Visibility::Unlisted, "Unlisted"),
                    (Visibility::Private, "Followers"),
                    (Visibility::Direct, "Direct"),
                ]
                key=|(visibility, _)| format!("{visibility:?}")
                let:entry
            >
                <option value=entry.0.as_str() selected=move || visibility.get() == entry.0>
                    {entry.1}
                </option>
            </For>
        </select>
    }
}
