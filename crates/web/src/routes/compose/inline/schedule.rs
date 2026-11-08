//! The schedule panel the caret next to the post button opens.
//!
//! A post is scheduled by absolute time only: "tomorrow at 9" is a convenience that
//! resolves to a clock time here, so what is stored is always one unambiguous value.

mod calendar;

use leptos::prelude::*;

use crate::state::clock_format;
use calendar::Calendar;

/// The panel, hung from the caret rather than from a control of its own: the answer
/// it sets is read by the post button's label, so the two read as one control.
#[component]
pub(super) fn SchedulePanel(
    /// `YYYY-MM-DDTHH:mm`, or empty for "send now".
    at: ReadSignal<String>,
    set_at: WriteSignal<String>,
    open: RwSignal<bool>,
) -> impl IntoView {
    // Mounted only while open, so this seeds from the current answer; the grid and
    // the fields edit a draft, and half-finished choices schedule nothing.
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
                        {move || format!("Schedule for {}", describe(&draft.get()))}
                    </button>
                </div>
            </div>
        </>
    }
}

/// One promise on the left, the clock time it means on the right.
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

/// The custom row: a month grid and two fields for the clock time.
#[component]
fn CustomRow(draft: ReadSignal<String>, set_draft: WriteSignal<String>) -> impl IntoView {
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
            // A date field that opens the grid: the value is what gets read back, and
            // the grid is the help, not the only way in.
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
                    draft=draft
                    set_draft=set_draft
                    close=Callback::new(move |()| calendar_open.set(false))
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
                    on:focus=move |event| select_all(&event)
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
                    on:focus=move |event| select_all(&event)
                    on:change=move |event| {
                        let (day_value, hour, _) = split(&draft.get_untracked());
                        write(day_value, hour, event_target_value(&event));
                    }
                />
                <span class="ml-auto text-[0.65rem] opacity-50">
                    {move || match clock_format::format() {
                        clock_format::ClockFormat::TwentyFour => "24-hour",
                        clock_format::ClockFormat::Twelve => "AM/PM",
                    }}
                </span>
            </div>
        </div>
    }
}

/// Selects the whole value, so the first keystroke replaces "09" instead of appending.
fn select_all(event: &web_sys::FocusEvent) {
    use wasm_bindgen::JsCast;
    if let Some(input) = event
        .target()
        .and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok())
    {
        input.select();
    }
}

/// `YYYY-MM-DD` and `HH`/`mm` out of `YYYY-MM-DDTHH:mm`.
pub(super) fn split(at: &str) -> (String, String, String) {
    let (day_value, clock) = at.split_once('T').unwrap_or((at, "09:00"));
    let (hour, minute) = clock.split_once(':').unwrap_or(("09", "00"));
    (day_value.to_owned(), hour.to_owned(), minute.to_owned())
}

/// "Send now", "Today, 09:00", "Tomorrow, 09:00" or "Sun 9, 09:00".
fn describe(at: &str) -> String {
    if at.trim().is_empty() {
        return "Send now".to_owned();
    }
    let clock = clock_format::format();
    let (value, hour, minute) = split(at);
    let hour_number = hour.parse::<u32>().unwrap_or(0);
    let minute_number = minute.parse::<u32>().unwrap_or(0);
    let label = if value == today(0) {
        "Today".to_owned()
    } else if value == today(1) {
        "Tomorrow".to_owned()
    } else {
        weekday_label(&value).unwrap_or(value)
    };
    format!("{label}, {}", clock.time(hour_number, minute_number))
}

/// The label under a day button: "Today", "Tomorrow", or "Mon 10".
fn day(offset: i64) -> (String, String) {
    // A day is 86 400 000 ms; two weeks of them stay far inside `f64`'s exact range.
    let shift = f64::from(i32::try_from(offset).unwrap_or(0)) * 86_400_000.0;
    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(
        js_sys::Date::now() + shift,
    ));
    let value = stamp(&date);
    let label = match offset {
        0 => "Today".to_owned(),
        1 => "Tomorrow".to_owned(),
        _ => weekday_label(&value).unwrap_or_else(|| value.clone()),
    };
    (value, label)
}

/// "Mon 10" for a stored `YYYY-MM-DD`, when this browser can name the weekday.
fn weekday_label(value: &str) -> Option<String> {
    let date = js_sys::Date::parse(value);
    if date.is_nan() {
        return None;
    }
    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(date));
    let weekday = date.to_locale_string(
        "en",
        &wasm_bindgen::JsValue::from_str("{\"weekday\":\"short\"}"),
    );
    Some(format!(
        "{} {}",
        weekday.as_string().unwrap_or_default(),
        date.get_date()
    ))
}

/// `YYYY-MM-DD` for `offset` days from today.
fn today(offset: i64) -> String {
    day(offset).0
}

/// `YYYY-MM-DDTHH:mm` a quarter-hour-rounded `minutes` from now.
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
    format!("{}T{hour:02}:00", today(days))
}

/// `YYYY-MM-DDTHH:mm` for `hour` o'clock on the next `weekday` (0 = Sunday).
fn next_weekday_at(weekday: u32, hour: u32) -> String {
    let now = js_sys::Date::new_0();
    let mut ahead = (i64::from(weekday) - i64::from(now.get_day())).rem_euclid(7);
    if ahead == 0 {
        ahead = 7;
    }
    in_days_at(ahead, hour)
}

/// The `YYYY-MM-DD` shape the panel works in.
fn stamp(date: &js_sys::Date) -> String {
    format!(
        "{:04}-{:02}-{:02}",
        date.get_full_year(),
        date.get_month() + 1,
        date.get_date()
    )
}
