//! The schedule panel the caret next to the post button opens.
//!
//! The calendar and the clock labels are the caller's: they need the reader's zone and the
//! chosen clock style. What is here is the panel, the quick answers, and the two time fields.

use leptos::prelude::*;

/// One promise on the left, the clock time it means on the right.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuickTime {
    pub label: String,
    pub when: String,
}

/// The panel, hung from the caret rather than from a control of its own: the answer
/// it sets is read by the post button's label, so the two read as one control.
#[component]
#[expect(
    clippy::too_many_arguments,
    reason = "the panel is one form: its fields and the calendar it embeds are all its own"
)]
pub fn SchedulePanel(
    /// `YYYY-MM-DDTHH:mm`, or empty for "send now".
    #[prop(into)]
    draft: Signal<String>,
    set_draft: WriteSignal<String>,
    set_at: WriteSignal<String>,
    open: RwSignal<bool>,
    /// The quick answers, with the clock value each stands for already resolved.
    quick: Vec<QuickTime>,
    /// Formats a `YYYY-MM-DDTHH:mm` the way this reader's clock shows it.
    #[prop(optional_no_strip)]
    describe: Option<Callback<String, String>>,
    /// "24-hour" or "AM/PM".
    #[prop(into)]
    clock: String,
    /// The month the grid currently shows, already named.
    #[prop(into)]
    month_label: Signal<String>,
    /// The grid's cells: `None` for the blank days before the first of the month.
    #[prop(into)]
    cells: Signal<Vec<Option<u32>>>,
    /// The day the current answer falls on.
    #[prop(into)]
    selected: Signal<String>,
    #[prop(optional_no_strip)] on_prev_month: Option<Callback<()>>,
    #[prop(optional_no_strip)] on_next_month: Option<Callback<()>>,
    /// Reports the day number chosen in the grid.
    #[prop(optional_no_strip)]
    on_day: Option<Callback<u32>>,
) -> impl IntoView {
    let close = Callback::new(move |()| open.set(false));

    view! {
        <>
            <div class="fixed inset-0 z-20" on:click=move |_| close.run(())></div>
            <div class="absolute top-full right-0 z-30 mt-2 w-80 rounded-box border border-base-300 bg-base-100 p-2 shadow-xl">
                <ul class="menu w-full gap-0 p-0">
                    <For each=move || quick.clone() key=|quick| quick.label.clone() let:quick>
                        <Quick quick=quick set_at=set_at close=close describe=describe/>
                    </For>
                </ul>

                <CustomRow
                    draft=draft
                    set_draft=set_draft
                    clock=clock
                    month_label=month_label
                    cells=cells
                    selected=selected
                    on_prev_month=on_prev_month
                    on_next_month=on_next_month
                    on_day=on_day
                />

                <div class="mt-3 flex gap-2 px-2 pb-1">
                    <button type="button" class="btn btn-ghost btn-sm" on:click=move |_| close.run(())>
                        "Cancel"
                    </button>
                    <button
                        type="button"
                        class="btn btn-primary btn-sm flex-1"
                        on:click=move |_| {
                            set_at.set(draft.get_untracked());
                            close.run(());
                        }
                    >
                        {move || {
                            let described = describe.map_or_else(
                                || draft.get(),
                                |describe| describe.run(draft.get()),
                            );
                            format!("Schedule for {described}")
                        }}
                    </button>
                </div>
            </div>
        </>
    }
}

/// One promise on the left, the clock time it means on the right.
#[component]
fn Quick(
    quick: QuickTime,
    set_at: WriteSignal<String>,
    close: Callback<()>,
    #[prop(optional_no_strip)] describe: Option<Callback<String, String>>,
) -> impl IntoView {
    let hint = if quick.when.is_empty() {
        String::new()
    } else {
        describe.map_or_else(
            || quick.when.clone(),
            |describe| describe.run(quick.when.clone()),
        )
    };
    let when = quick.when;
    view! {
        <li>
            <button
                type="button"
                class="flex w-full items-center justify-between gap-2"
                on:click=move |_| {
                    set_at.set(when.clone());
                    close.run(());
                }
            >
                <span>{quick.label}</span>
                <span class="text-xs opacity-60">{hint}</span>
            </button>
        </li>
    }
}

/// The custom row: a month grid and two fields for the clock time.
#[component]
#[expect(
    clippy::too_many_arguments,
    reason = "the row is the panel's form: the grid and its navigation are part of it"
)]
fn CustomRow(
    #[prop(into)] draft: Signal<String>,
    set_draft: WriteSignal<String>,
    #[prop(into)] clock: String,
    #[prop(into)] month_label: Signal<String>,
    #[prop(into)] cells: Signal<Vec<Option<u32>>>,
    #[prop(into)] selected: Signal<String>,
    #[prop(optional_no_strip)] on_prev_month: Option<Callback<()>>,
    #[prop(optional_no_strip)] on_next_month: Option<Callback<()>>,
    #[prop(optional_no_strip)] on_day: Option<Callback<u32>>,
) -> impl IntoView {
    let calendar_open = RwSignal::new(false);
    // The fields are text, so "9" is read as nine o'clock and anything past the end
    // of the clock is clamped here rather than sent to the instance as nonsense.
    let write = move |day_value: String, hour: String, minute: String| {
        let clean = |value: &str, max: u32| {
            let digits: String = value.chars().filter(char::is_ascii_digit).collect();
            format!("{:02}", digits.parse::<u32>().unwrap_or(0).min(max))
        };
        set_draft.set(format!(
            "{day_value}T{}:{}",
            clean(&hour, 23),
            clean(&minute, 59)
        ));
    };

    view! {
        <div class="divider my-1 text-xs opacity-60">"Or pick a time"</div>

        <div class="px-2">
            <button
                type="button"
                class="input input-bordered input-sm mb-2 flex w-full items-center justify-between text-left"
                on:click=move |_| calendar_open.update(|shown| *shown = !*shown)
                aria-expanded=move || calendar_open.get().to_string()
                aria-label="Pick a day"
            >
                <span>{move || split(&draft.get()).0}</span>
                <span class="text-[0.6rem] opacity-60">"▼"</span>
            </button>

            <Show when=move || calendar_open.get()>
                <Calendar
                    cells=cells
                    month_label=month_label
                    selected=selected
                    on_prev_month=on_prev_month
                    on_next_month=on_next_month
                    on_day=Some(Callback::new(move |day: u32| {
                        if let Some(on_day) = on_day {
                            on_day.run(day);
                        }
                        calendar_open.set(false);
                    }))
                />
            </Show>

            <div class="flex items-center gap-2 text-xs">
                <input
                    class="input input-bordered input-sm w-14 text-center tabular-nums"
                    inputmode="numeric"
                    maxlength="2"
                    placeholder="HH"
                    aria-label="Hour"
                    prop:value=move || split(&draft.get()).1
                    on:change=move |event| {
                        let (day_value, _, minute) = split(&draft.get_untracked());
                        write(day_value, event_target_value(&event), minute);
                    }
                />
                <span class="opacity-50">":"</span>
                <input
                    class="input input-bordered input-sm w-14 text-center tabular-nums"
                    inputmode="numeric"
                    maxlength="2"
                    placeholder="mm"
                    aria-label="Minute"
                    prop:value=move || split(&draft.get()).2
                    on:change=move |event| {
                        let (day_value, hour, _) = split(&draft.get_untracked());
                        write(day_value, hour, event_target_value(&event));
                    }
                />
                <span class="ml-auto text-[0.65rem] opacity-50">{clock}</span>
            </div>
        </div>
    }
}

/// A month grid, in place of a native picker.
#[component]
fn Calendar(
    #[prop(into)] cells: Signal<Vec<Option<u32>>>,
    #[prop(into)] month_label: Signal<String>,
    #[prop(into)] selected: Signal<String>,
    #[prop(optional_no_strip)] on_prev_month: Option<Callback<()>>,
    #[prop(optional_no_strip)] on_next_month: Option<Callback<()>>,
    #[prop(optional_no_strip)] on_day: Option<Callback<u32>>,
) -> impl IntoView {
    view! {
        <div class="mb-2 rounded-box border border-base-300 p-2">
            <div class="mb-1 flex items-center justify-between">
                <button type="button" class="btn btn-ghost btn-xs px-1" aria-label="Previous month"
                    on:click=move |_| {
                        if let Some(on_prev_month) = on_prev_month {
                            on_prev_month.run(());
                        }
                    }>"‹"</button>
                <span class="text-xs font-medium">{move || month_label.get()}</span>
                <button type="button" class="btn btn-ghost btn-xs px-1" aria-label="Next month"
                    on:click=move |_| {
                        if let Some(on_next_month) = on_next_month {
                            on_next_month.run(());
                        }
                    }>"›"</button>
            </div>

            <div class="grid grid-cols-7 gap-1 text-center text-[0.6rem] opacity-50">
                <span>"M"</span><span>"T"</span><span>"W"</span><span>"T"</span>
                <span>"F"</span><span>"S"</span><span>"S"</span>
            </div>
            <div class="grid grid-cols-7 gap-1">
                <For each=move || cells.get() key=|cell| cell.unwrap_or(0) let:cell>
                    {match cell {
                        Some(day) => view! {
                            <button
                                type="button"
                                class="btn btn-ghost h-7 min-h-0 rounded-md px-0 text-xs"
                                class:bg-primary=move || selected.get().ends_with(&format!("-{day:02}"))
                                class:text-primary-content=move || selected.get().ends_with(&format!("-{day:02}"))
                                on:click=move |_| {
                                    if let Some(on_day) = on_day {
                                        on_day.run(day);
                                    }
                                }
                            >
                                {day}
                            </button>
                        }.into_any(),
                        None => view! { <span></span> }.into_any(),
                    }}
                </For>
            </div>
        </div>
    }
}

/// `YYYY-MM-DD` and `HH`/`mm` out of `YYYY-MM-DDTHH:mm`.
#[must_use]
pub fn split(at: &str) -> (String, String, String) {
    let (day_value, clock) = at.split_once('T').unwrap_or((at, "09:00"));
    let (hour, minute) = clock.split_once(':').unwrap_or(("09", "00"));
    (day_value.to_owned(), hour.to_owned(), minute.to_owned())
}
