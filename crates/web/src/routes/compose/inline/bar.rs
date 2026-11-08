//! The row under the textarea: visibility, when to send, the counter, the button.

use leptos::prelude::*;
use nodus_instance::model::Visibility;

use super::{LIMIT, VisibilityPicker, When};

/// Every control that describes *how* the post goes out, kept apart from the box that
/// holds the text so neither half outgrows what one screen can read.
#[component]
pub(super) fn ComposerBar(
    text: ReadSignal<String>,
    visibility: ReadSignal<Visibility>,
    set_visibility: WriteSignal<Visibility>,
    when: ReadSignal<When>,
    set_when: WriteSignal<When>,
    busy: ReadSignal<bool>,
    on_submit: Callback<()>,
) -> impl IntoView {
    let count = move || text.get().chars().count();
    view! {
        <div class="flex flex-wrap items-center gap-2">
            <VisibilityPicker visibility=visibility set_visibility=set_visibility/>

            <select
                class="select select-ghost select-xs w-auto"
                aria-label="When to send"
                // Bound to the signal both ways: the value returns to "Send now" once the
                // post has gone out, and the reader sees that it did.
                prop:value=move || {
                    When::ALL
                        .iter()
                        .position(|option| *option == when.get())
                        .unwrap_or(0)
                        .to_string()
                }
                on:change=move |event| {
                    let index = event_target_value(&event).parse::<usize>().unwrap_or(0);
                    set_when.set(When::ALL.get(index).copied().unwrap_or(When::Now));
                }
            >
                // Written out rather than generated: four fixed presets, and a static
                // option is the only shape the view macro accepts inside a select.
                <option value="0">"Send now"</option>
                <option value="1">"In 30 minutes"</option>
                <option value="2">"In 2 hours"</option>
                <option value="3">"Tomorrow at 9"</option>
            </select>

            <span
                class="text-xs tabular-nums"
                class=("opacity-50", move || count() <= LIMIT)
                class=("text-error", move || count() > LIMIT)
                title="The instance refuses a post longer than this"
            >
                {move || format!("{}/{LIMIT}", count())}
            </span>

            <button
                class="btn btn-primary btn-sm gap-1"
                disabled=move || busy.get() || text.get().trim().is_empty()
                on:click=move |_| on_submit.run(())
            >
                {move || if busy.get() { "Posting…" } else { "Post" }}
            </button>
        </div>
    }
}
