//! The poll controls: two to four options, a duration, and two voting rules.
//!
//! The draft lives in the composer, so every field arrives as the signal that holds it and
//! every edit writes back through the same signal. The one thing the view cannot do is put
//! the caret in a row it just added — that needs the document — so it reports the new index.

use leptos::prelude::*;

/// How long a poll may stay open, in seconds, with the label the select shows.
pub const DURATIONS: [(u32, &str); 4] = [
    (86_400, "1 day"),
    (3 * 86_400, "3 days"),
    (7 * 86_400, "1 week"),
    (30 * 86_400, "30 days"),
];

/// The poll controls, shown only while the poll is part of the post.
#[component]
pub fn PollEditor(
    open: RwSignal<bool>,
    options: RwSignal<Vec<String>>,
    expires_in: RwSignal<u32>,
    multiple: RwSignal<bool>,
    hide_totals: RwSignal<bool>,
    /// Reports the index of the option just appended, so the caller can focus it.
    #[prop(optional_no_strip)]
    on_add: Option<Callback<usize>>,
) -> impl IntoView {
    view! {
        <Show when=move || open.get()>
            <div class="flex flex-col gap-2 rounded-box border border-base-300 p-2">
                <div class="flex items-center justify-between">
                    <span class="text-xs font-semibold uppercase tracking-wide opacity-50">
                        "Poll"
                    </span>
                    <button
                        type="button"
                        class="btn btn-ghost btn-xs"
                        title="Remove the poll"
                        on:click=move |_| {
                            options.set(vec![String::new(), String::new()]);
                            open.set(false);
                        }
                    >
                        "Remove"
                    </button>
                </div>

                <Options options=options/>
                <PollSettings
                    options=options
                    expires_in=expires_in
                    multiple=multiple
                    hide_totals=hide_totals
                    on_add=on_add
                />
            </div>
        </Show>
    }
}

/// The options, one row each.
#[component]
fn Options(options: RwSignal<Vec<String>>) -> impl IntoView {
    let update = move |index: usize, value: String| {
        options.update(|list| {
            if let Some(slot) = list.get_mut(index) {
                *slot = value;
            }
        });
    };
    let indices = Signal::derive(move || (0..options.get().len()).collect::<Vec<_>>());

    view! {
        <For each=move || indices.get() key=|index| *index let:index>
            <div class="flex items-center gap-1">
                <input
                    class="input input-bordered input-sm w-full"
                    id=move || format!("poll-option-{index}")
                    placeholder=move || format!("Option {}", index + 1)
                    aria-label=move || format!("Poll option {}", index + 1)
                    prop:value=move || options.get().get(index).cloned().unwrap_or_default()
                    on:input=move |event| update(index, event_target_value(&event))
                />
                <button
                    type="button"
                    class="btn btn-ghost btn-xs"
                    title="Remove this option"
                    disabled=move || options.get().len() <= 2
                    on:click=move |_| {
                        options.update(|list| {
                            if list.len() > 2 {
                                list.remove(index);
                            }
                        });
                    }
                >
                    "×"
                </button>
            </div>
        </For>
    }
}

/// How long the poll runs, and who may pick how many answers.
#[component]
fn PollSettings(
    options: RwSignal<Vec<String>>,
    expires_in: RwSignal<u32>,
    multiple: RwSignal<bool>,
    hide_totals: RwSignal<bool>,
    #[prop(optional_no_strip)] on_add: Option<Callback<usize>>,
) -> impl IntoView {
    view! {
        <div class="flex flex-wrap items-center gap-2">
            <button
                type="button"
                class="btn btn-outline btn-xs gap-1 border-dashed"
                title=move || {
                    if options.get().len() >= 4 {
                        "A poll holds at most four options"
                    } else {
                        "Add another option"
                    }
                }
                disabled=move || options.get().len() >= 4
                on:click=move |_| {
                    let added = options.with_untracked(Vec::len);
                    options.update(|list| list.push(String::new()));
                    if let Some(on_add) = on_add {
                        on_add.run(added);
                    }
                }
            >
                "+"
                "Add option"
            </button>

            <select
                class="select select-bordered select-sm w-fit"
                aria-label="How long the poll stays open"
                prop:value=move || {
                    let held = expires_in.get();
                    DURATIONS
                        .iter()
                        .position(|(seconds, _)| *seconds == held)
                        .unwrap_or(0)
                        .to_string()
                }
                on:change=move |event| {
                    let index = event_target_value(&event).parse::<usize>().unwrap_or(0);
                    expires_in.set(DURATIONS.get(index).map_or(86_400, |(seconds, _)| *seconds));
                }
            >
                {DURATIONS
                    .iter()
                    .enumerate()
                    .map(|(index, (_, label))| {
                        view! { <option value=index.to_string()>{*label}</option> }
                    })
                    .collect::<Vec<_>>()}
            </select>

            <label class="label cursor-pointer gap-1 p-0 text-xs">
                <input
                    type="checkbox"
                    class="checkbox checkbox-xs"
                    prop:checked=move || multiple.get()
                    on:change=move |event| multiple.set(event_target_checked(&event))
                />
                "Several answers"
            </label>

            <label class="label cursor-pointer gap-1 p-0 text-xs">
                <input
                    type="checkbox"
                    class="checkbox checkbox-xs"
                    prop:checked=move || hide_totals.get()
                    on:change=move |event| hide_totals.set(event_target_checked(&event))
                />
                "Hide counts until it closes"
            </label>
        </div>
    }
}
