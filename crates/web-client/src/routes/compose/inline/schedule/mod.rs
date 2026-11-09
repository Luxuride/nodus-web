//! The schedule panel the caret next to the post button opens.
//!
//! A post is scheduled by absolute time only: "tomorrow at 9" is a convenience that
//! resolves to a clock time here, so what is stored is always one unambiguous value.
//!
//! The panel's markup is shared. What stays in this crate is the reader's clock: the quick
//! answers, the month grid, and the label the button shows all depend on today's date here.

use leptos::prelude::*;
use nodus_pages::pages::compose::inline::schedule::{QuickTime, SchedulePanel as Panel};

use crate::state::clock_format;

pub(super) use nodus_pages::pages::compose::inline::schedule::split;

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
    let today = js_sys::Date::new_0();
    let (shown, set_shown) = signal((today.get_full_year(), today.get_month()));

    let cells = Signal::derive(move || {
        let (year, month) = shown.get();
        let month = i32::try_from(month).unwrap_or(0);
        let first = js_sys::Date::new_with_year_month_day(year, month, 1);
        let mut cells: Vec<Option<u32>> =
            vec![None; usize::try_from(first.get_day().min(6)).unwrap_or(0)];
        cells.extend((1..=last_day(year, month)).map(Some));
        cells
    });

    let month_label = Signal::derive(move || {
        let (year, month) = shown.get();
        let name =
            js_sys::Date::new_with_year_month_day(year, i32::try_from(month).unwrap_or(0), 1);
        let month_name = name.to_locale_string(
            "en",
            &wasm_bindgen::JsValue::from_str("{\"month\":\"long\"}"),
        );
        format!("{} {year}", month_name.as_string().unwrap_or_default())
    });

    let selected = Signal::derive(move || split(&draft.get()).0);
    let clock = match clock_format::format() {
        clock_format::ClockFormat::TwentyFour => "24-hour",
        clock_format::ClockFormat::Twelve => "AM/PM",
    };
    let quick = vec![
        QuickTime {
            label: "Send now".to_owned(),
            when: String::new(),
        },
        QuickTime {
            label: "In 30 minutes".to_owned(),
            when: in_minutes(30),
        },
        QuickTime {
            label: "In 1 hour".to_owned(),
            when: in_minutes(60),
        },
        QuickTime {
            label: "In 2 hours".to_owned(),
            when: in_minutes(120),
        },
        QuickTime {
            label: "Tomorrow at 9:00".to_owned(),
            when: in_days_at(1, 9),
        },
        QuickTime {
            label: "Next Monday at 9:00".to_owned(),
            when: next_weekday_at(1, 9),
        },
    ];

    let on_day = Callback::new(move |day: u32| {
        let (year, month) = shown.get_untracked();
        let (_, hour, minute) = split(&draft.get_untracked());
        let month = month + 1;
        set_draft.set(format!("{year:04}-{month:02}-{day:02}T{hour}:{minute}"));
    });

    view! {
        <Panel
            draft=Signal::derive(move || draft.get())
            set_draft=set_draft
            set_at=set_at
            open=open
            quick=quick
            describe=Some(Callback::new(describe))
            clock=clock
            month_label=month_label
            cells=cells
            selected=selected
            on_prev_month=Some(Callback::new(move |()| {
                set_shown.update(|(year, month)| {
                    if *month == 0 { *month = 11; *year -= 1; } else { *month -= 1; }
                });
            }))
            on_next_month=Some(Callback::new(move |()| {
                set_shown.update(|(year, month)| {
                    if *month == 11 { *month = 0; *year += 1; } else { *month += 1; }
                });
            }))
            on_day=Some(on_day)
        />
    }
}

/// The last day number of a 0-based month: day 0 of the next month is its last day.
fn last_day(year: u32, month: i32) -> u32 {
    js_sys::Date::new_with_year_month_day(year, month + 1, 0).get_date()
}

/// "Send now", "Today, 09:00", "Tomorrow, 09:00" or "Sun 9, 09:00".
fn describe(at: String) -> String {
    if at.trim().is_empty() {
        return "Send now".to_owned();
    }
    let clock = clock_format::format();
    let (value, hour, minute) = split(&at);
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
