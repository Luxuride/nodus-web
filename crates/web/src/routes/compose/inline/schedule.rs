//! When a post should go out, in the shape Discord's scheduler uses: one panel of answers
//! that are already the answer ("In 1 hour — 15:05"), and a custom row underneath for the
//! times none of them cover.
//!
//! Not a native date picker. That widget is a browser-and-locale-shaped calendar that
//! opens as a modal of its own, hides the time behind a second tab in some engines, and
//! returns a string whose zone rules differ by browser — a lot of behaviour to inherit for
//! a field most posts never touch. Three selects say the same thing where the composer
//! already is, in 24-hour time, in the reader's zone.

use leptos::prelude::*;

/// How many days ahead the day list offers. Two weeks covers "next Monday" without
/// turning the list into a calendar.
const DAYS: usize = 14;

/// Minutes past the hour the picker offers: the quarter hours people schedule at.
const MINUTES: [&str; 4] = ["00", "15", "30", "45"];

const WEEKDAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

/// A `YYYY-MM-DD` for the local day `offset` days from today, and the label to show.
fn day(offset: i64) -> (String, String) {
    let date = local_days_from_now(offset);
    let value = format!(
        "{:04}-{:02}-{:02}",
        date.get_full_year(),
        date.get_month() + 1,
        date.get_date()
    );
    let label = match offset {
        0 => "Today".to_owned(),
        1 => "Tomorrow".to_owned(),
        _ => {
            // `get_day` is Sunday-first, which is the order `WEEKDAYS` is written in.
            let weekday = WEEKDAYS[usize::try_from(date.get_day()).unwrap_or(0) % 7];
            format!("{weekday} {}", date.get_date())
        }
    };
    (value, label)
}

fn local_days_from_now(offset: i64) -> js_sys::Date {
    let now = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(js_sys::Date::now()));
    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(now.get_time()));
    date.set_date(now.get_date() + u32::try_from(offset).unwrap_or(0));
    date
}

/// Splits `YYYY-MM-DDTHH:mm` into (day, hour, minute), falling back to nine today when the
/// value is empty or not that shape.
fn split(value: &str) -> (String, String, String) {
    let mut parts = value.split('T');
    let (Some(day_part), Some(time_part)) = (parts.next(), parts.next()) else {
        return ((day(0)).0, "09".to_owned(), "00".to_owned());
    };
    let mut clock = time_part.split(':');
    let (Some(hour), Some(minute)) = (clock.next(), clock.next()) else {
        return ((day(0)).0, "09".to_owned(), "00".to_owned());
    };
    if day_part.len() != 10 || hour.len() != 2 || minute.len() != 2 {
        return ((day(0)).0, "09".to_owned(), "00".to_owned());
    }
    (day_part.to_owned(), hour.to_owned(), minute.to_owned())
}

/// What the control says: the promise the post is making.
fn describe(at: &str) -> String {
    if at.trim().is_empty() {
        return "Send now".to_owned();
    }
    let (value, hour, minute) = split(at);
    let label = (0..DAYS)
        .map(|offset| day(i64::try_from(offset).unwrap_or(0)))
        .find(|(candidate, _)| *candidate == value)
        .map_or_else(|| value.clone(), |(_, label)| label);
    format!("{label}, {hour}:{minute}")
}

/// The schedule panel the caret next to the post button opens.
///
/// It hangs from the caret rather than from a control of its own: the answer it
/// sets is read by the post button's label, so the two are one control in a
/// reader's head, and putting the panel under the button keeps them together.
#[component]
pub(super) fn SchedulePanel(
    /// `YYYY-MM-DDTHH:mm`, or empty for "send now".
    at: ReadSignal<String>,
    set_at: WriteSignal<String>,
    open: RwSignal<bool>,
) -> impl IntoView {
    // Mounted only while the panel is open, so this seeds it from the current answer;
    // the custom row edits a draft, so half-finished choices schedule nothing.
    let (draft, set_draft) = signal(at.get_untracked());
    let close = Callback::new(move |()| open.set(false));

    view! {
        <>
            <div class="fixed inset-0 z-20" on:click=move |_| close.run(())></div>
            <div class="absolute top-full right-0 z-30 mt-2 w-80 rounded-box border border-base-300 bg-base-100 p-2 shadow-xl">
                    <ul class="menu w-full gap-0 p-0">
                        <Quick label="Send now" when=String::new() set_at=set_at close=close/>
                        <Quick label="In 30 minutes" when=in_minutes(30) set_at=set_at close=close/>
                        <Quick label="In 1 hour" when=in_minutes(60) set_at=set_at close=close/>
                        <Quick label="In 2 hours" when=in_minutes(120) set_at=set_at close=close/>
                        <Quick label="Tomorrow at 9:00" when=in_days_at(1, 9) set_at=set_at close=close/>
                        <Quick label="Next Monday at 9:00" when=next_weekday_at(1, 9) set_at=set_at close=close/>
                    </ul>

                    <CustomRow draft=draft set_draft=set_draft/>

                    <div class="mt-3 flex gap-2 px-2 pb-1">
                        <button
                            type="button"
                            class="btn btn-ghost btn-sm"
                            on:click=move |_| close.run(())
                        >
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
                            {move || format!("Schedule for {}", describe(&draft.get()))}
                        </button>
                    </div>
                </div>
        </>
    }
}

/// The custom row: a day, an hour and a minute, as three selects.
#[component]
fn CustomRow(draft: ReadSignal<String>, set_draft: WriteSignal<String>) -> impl IntoView {
    // Built outside the view: a turbofish inside a `view!` is read as an attribute key.
    // The day list depends on today, so it is computed when this panel is built.
    let days = StoredValue::new(
        (0..DAYS)
            .map(|offset| day(i64::try_from(offset).unwrap_or(0)))
            .collect::<Vec<_>>(),
    );
    let hours = StoredValue::new((0..24).map(|hour| format!("{hour:02}")).collect::<Vec<_>>());
    let minutes = StoredValue::new(MINUTES.map(str::to_owned).to_vec());

    // One answer, assembled in one place, so the three selects cannot disagree.
    let write = move |day_value: String, hour: String, minute: String| {
        set_draft.set(format!("{day_value}T{hour}:{minute}"));
    };

    view! {
        <div class="divider my-1 text-xs opacity-60">"Or pick a time"</div>

                    <div class="px-2">
                        <select
                            class="select select-bordered select-sm mb-2 w-full"
                            aria-label="Day"
                            prop:value=move || split(&draft.get()).0
                            on:change=move |event| {
                                let (_, hour, minute) = split(&draft.get_untracked());
                                write(event_target_value(&event), hour, minute);
                            }
                        >
                            <For
                                each=move || days.get_value()
                                key=|(value, _)| value.clone()
                                let:entry
                            >
                                <option value=entry.0.clone()>{entry.1.clone()}</option>
                            </For>
                        </select>

                        <div class="flex gap-2">
                            <select
                                class="select select-bordered select-sm flex-1"
                                aria-label="Hour"
                                prop:value=move || split(&draft.get()).1
                                on:change=move |event| {
                                    let (day_value, _, minute) = split(&draft.get_untracked());
                                    write(day_value, event_target_value(&event), minute);
                                }
                            >
                                <For
                                    each=move || hours.get_value()
                                    key=|hour| hour.clone()
                                    let:hour
                                >
                                    <option value=hour.clone()>{hour.clone()}</option>
                                </For>
                            </select>
                            <select
                                class="select select-bordered select-sm w-24"
                                aria-label="Minute"
                                prop:value=move || split(&draft.get()).2
                                on:change=move |event| {
                                    let (day_value, hour, _) = split(&draft.get_untracked());
                                    write(day_value, hour, event_target_value(&event));
                                }
                            >
                                <For each=move || minutes.get_value() key=|minute| minute.clone() let:minute>
                                    <option value=minute.clone()>{minute.clone()}</option>
                                </For>
                            </select>
                        </div>
                    </div>

    }
}

/// One row of the panel: the promise on the left, the clock time it means on the right.
#[component]
fn Quick(
    label: &'static str,
    when: String,
    set_at: WriteSignal<String>,
    close: Callback<()>,
) -> impl IntoView {
    let clock = when.clone();
    let hint = if clock.is_empty() {
        String::new()
    } else {
        describe(&clock)
    };
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
                <span>{label}</span>
                <span class="text-xs opacity-60">{hint}</span>
            </button>
        </li>
    }
}

/// `YYYY-MM-DDTHH:mm` for `minutes` from now, rounded up to the next quarter hour.
fn in_minutes(minutes: u32) -> String {
    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(
        js_sys::Date::now() + f64::from(minutes) * 60_000.0,
    ));
    let rounded = date.get_minutes().div_ceil(15) * 15;
    date.set_hours(date.get_hours() + rounded / 60);
    date.set_minutes(rounded % 60);
    stamp(&date)
}

/// `YYYY-MM-DDTHH:mm` for `hour` o'clock, `days` days from today.
fn in_days_at(days: i64, hour: u32) -> String {
    let (value, _) = day(days);
    format!("{value}T{hour:02}:00")
}

/// `YYYY-MM-DDTHH:mm` for `hour` o'clock on the next `weekday` (0 = Sunday).
fn next_weekday_at(weekday: u32, hour: u32) -> String {
    let today = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(js_sys::Date::now()));
    let mut ahead = (i64::from(weekday) - i64::from(today.get_day())).rem_euclid(7);
    if ahead == 0 {
        ahead = 7;
    }
    in_days_at(ahead, hour)
}

/// The `YYYY-MM-DDTHH:mm` shape the panel works in.
fn stamp(date: &js_sys::Date) -> String {
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}",
        date.get_full_year(),
        date.get_month() + 1,
        date.get_date(),
        date.get_hours(),
        date.get_minutes()
    )
}
