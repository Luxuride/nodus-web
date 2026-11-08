use leptos::prelude::*;

/// The last day number of a 0-based month: day 0 of the next month is its last day.
fn last_day(year: u32, month: i32) -> u32 {
    js_sys::Date::new_with_year_month_day(year, month + 1, 0).get_date()
}

/// A month grid, in place of a native picker.
///
/// The native control is a browser dialog that looks nothing like the panel and
/// cannot show which day the current answer is; a grid can, and it can grey out
/// the days that have already passed.
#[component]
pub(super) fn Calendar(
    /// `YYYY-MM-DDTHH:mm`.
    draft: ReadSignal<String>,
    set_draft: WriteSignal<String>,
) -> impl IntoView {
    let today = js_sys::Date::new_0();
    let (shown, set_shown) = signal((today.get_full_year(), today.get_month()));

    let keep_time = move || {
        let (_, hour, minute) = super::split(&draft.get_untracked());
        (hour, minute)
    };

    let cells = move || {
        let (year, month) = shown.get();
        let first = js_sys::Date::new_with_year_month_day(year, i32::try_from(month).unwrap_or(0), 1);
        let mut cells: Vec<Option<u32>> =
            vec![None; usize::try_from(first.get_day().min(6)).unwrap_or(0)];
        cells.extend((1..=last_day(year, i32::try_from(month).unwrap_or(0))).map(Some));
        cells
    };

    view! {
        <div class="mb-2 rounded-box border border-base-300 p-2">
            <div class="mb-1 flex items-center justify-between">
                <button type="button" class="btn btn-ghost btn-xs px-1" aria-label="Previous month"
                    on:click=move |_| set_shown.update(|(year, month)| {
                        if *month == 0 { *month = 11; *year -= 1; } else { *month -= 1; }
                    })>"‹"</button>
                <span class="text-xs font-medium">
                    {move || {
                        let (year, month) = shown.get();
                        let name = js_sys::Date::new_with_year_month_day(year, i32::try_from(month).unwrap_or(0), 1);
                        let month_name = name.to_locale_string(
                            "en",
                            &wasm_bindgen::JsValue::from_str("{\"month\":\"long\"}"),
                        );
                        format!("{} {year}", month_name.as_string().unwrap_or_default())
                    }}
                </span>
                <button type="button" class="btn btn-ghost btn-xs px-1" aria-label="Next month"
                    on:click=move |_| set_shown.update(|(year, month)| {
                        if *month == 11 { *month = 0; *year += 1; } else { *month += 1; }
                    })>"›"</button>
            </div>

            <div class="grid grid-cols-7 gap-1 text-center text-[0.6rem] opacity-50">
                <span>"M"</span><span>"T"</span><span>"W"</span><span>"T"</span>
                <span>"F"</span><span>"S"</span><span>"S"</span>
            </div>
            <div class="grid grid-cols-7 gap-1">
                <For each=cells key=|cell| cell.unwrap_or(0) let:cell>
                    {match cell {
                        Some(day) => {
                            let value = move || {
                                let (year, month) = shown.get();
                                format!("{year:04}-{:02}-{day:02}", month + 1)
                            };
                            let chosen = value;
                            view! {
                                <button
                                    type="button"
                                    class="btn btn-ghost h-7 min-h-0 rounded-md px-0 text-xs"
                                    class:bg-primary=move || super::split(&draft.get()).0 == value()
                                    class:text-primary-content=move || super::split(&draft.get()).0 == value()
                                    on:click=move |_| {
                                        let (hour, minute) = keep_time();
                                        set_draft.set(format!("{}T{hour}:{minute}", chosen()));
                                    }
                                >
                                    {day}
                                </button>
                            }.into_any()
                        }
                        None => view! { <span></span> }.into_any(),
                    }}
                </For>
            </div>
        </div>
    }
}
