//! A poll, as the composer builds one.
//!
//! Kept apart from the composer body because it is a small form of its own: two to four
//! options, how long they stay open, and whether a voter may pick more than one. The
//! composer owns the state so the poll survives a re-render of this component, and this
//! module only renders it and reports edits upward — a component that owned the options
//! would lose them the moment the panel closed.

use leptos::prelude::*;
use nodus_instance::model::PollDraft;

/// How long a poll may stay open, in seconds, with the label the select shows.
pub(super) const DURATIONS: [(u32, &str); 4] = [
    (86_400, "1 day"),
    (3 * 86_400, "3 days"),
    (7 * 86_400, "1 week"),
    (30 * 86_400, "30 days"),
];

/// The poll's answer as the wire wants it, or `None` when it is not one yet.
///
/// Fewer than two non-empty options is not a poll: the instance rejects it, and asking a
/// question with one answer is a statement. Returning `None` here keeps that rule in one
/// place instead of inside the submit handler.
#[must_use]
pub(super) fn as_draft(
    open: bool,
    options: &[String],
    expires_in: u32,
    multiple: bool,
    hide_totals: bool,
) -> Option<PollDraft> {
    if !open {
        return None;
    }
    let filled: Vec<String> = options
        .iter()
        .map(|option| option.trim().to_owned())
        .filter(|option| !option.is_empty())
        .collect();
    if filled.len() < 2 {
        return None;
    }
    Some(
        PollDraft::new(filled, expires_in)
            .multiple(multiple)
            .hide_totals(hide_totals),
    )
}

/// The poll controls, shown only while the poll is part of the post.
#[component]
pub(super) fn PollEditor(
    open: RwSignal<bool>,
    options: RwSignal<Vec<String>>,
    expires_in: RwSignal<u32>,
    multiple: RwSignal<bool>,
    hide_totals: RwSignal<bool>,
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
    // Precomputed outside `view!`: a turbofish inside the macro is parsed as an attribute
    // key, which fails with a message about the `move` keyword two lines away.
    let indices = Signal::derive(move || (0..options.get().len()).collect::<Vec<_>>());

    view! {
        <For each=move || indices.get() key=|index| *index let:index>
            <div class="flex items-center gap-1">
                <input
                    class="input input-bordered input-sm w-full"
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
) -> impl IntoView {
    view! {
        <div class="flex flex-wrap items-center gap-2">
            <button
                type="button"
                class="btn btn-outline btn-xs gap-1 border-dashed"
                title="A poll holds at most four options"
                disabled=move || options.get().len() >= 4
                on:click=move |_| options.update(|list| list.push(String::new()))
            >
                "+"
                "Add option"
            </button>

            <select
                class="select select-bordered select-sm w-fit"
                aria-label="How long the poll stays open"
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
