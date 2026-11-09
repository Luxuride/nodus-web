use leptos::prelude::*;

use crate::state::clock_format::{self, ClockFormat};

/// How this reader writes a clock time.
///
/// It sits with the profile because it is about the reader's habit, not about the
/// instance: 24-hour is the default, and AM/PM is a choice they make once.
#[component]
pub fn ClockCard() -> impl IntoView {
    let chosen = clock_format::signal();
    let current = move || chosen.map_or(ClockFormat::TwentyFour, |signal| signal.get());

    let pick = move |format: ClockFormat| clock_format::set(format);

    view! {
        <nodus_view::components::panel::Panel
            title="Clock"
            description="Times in the composer and the schedule panel follow this. Nothing is sent to the instance either way."
        >
                <div class="flex gap-2">
                    <button
                        type="button"
                        class="btn btn-sm"
                        class:btn-primary=move || current() == ClockFormat::TwentyFour
                        class:btn-outline=move || current() != ClockFormat::TwentyFour
                        on:click=move |_| pick(ClockFormat::TwentyFour)
                    >
                        "24-hour"
                    </button>
                    <button
                        type="button"
                        class="btn btn-sm"
                        class:btn-primary=move || current() == ClockFormat::Twelve
                        class:btn-outline=move || current() != ClockFormat::Twelve
                        on:click=move |_| pick(ClockFormat::Twelve)
                    >
                        "AM / PM"
                    </button>
                    <span class="ml-auto self-center text-xs opacity-60 tabular-nums">
                        {move || current().time(21, 30)}
                    </span>
                </div>
        </nodus_view::components::panel::Panel>
    }
}
