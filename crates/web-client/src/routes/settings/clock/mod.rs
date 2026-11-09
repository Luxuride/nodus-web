use leptos::prelude::*;
use nodus_view::pages::settings::profile::{ClockCard as SharedClockCard, ClockStyle};

use crate::state::clock_format::{self, ClockFormat};

/// How this reader writes a clock time.
///
/// It sits with the profile because it is about the reader's habit, not about the
/// instance: 24-hour is the default, and AM/PM is a choice they make once.
#[component]
pub fn ClockCard() -> impl IntoView {
    let chosen = clock_format::signal();
    let current = move || chosen.map_or(ClockFormat::TwentyFour, |signal| signal.get());
    let selected = Signal::derive(move || style(current()));
    let sample = Signal::derive(move || current().time(21, 30));

    view! {
        <SharedClockCard
            selected=selected
            sample=sample
            on_select=Some(Callback::new(move |style: ClockStyle| {
                clock_format::set(match style {
                    ClockStyle::TwentyFour => ClockFormat::TwentyFour,
                    ClockStyle::Twelve => ClockFormat::Twelve,
                });
            }))
        />
    }
}

fn style(format: ClockFormat) -> ClockStyle {
    match format {
        ClockFormat::TwentyFour => ClockStyle::TwentyFour,
        ClockFormat::Twelve => ClockStyle::Twelve,
    }
}
