//! How this reader writes a clock time: 24-hour, or with AM/PM.
//!
//! A preference, not a locale guess: the same machine is used by people who read
//! "21:30" and by people who read "9:30 PM", and a browser locale is not a promise
//! about the reader's habit.

use leptos::prelude::*;

const KEY: &str = "nodus-clock/1";

thread_local! {
    static FORMAT: std::cell::RefCell<Option<RwSignal<ClockFormat>>> = const { std::cell::RefCell::new(None) };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockFormat {
    TwentyFour,
    Twelve,
}

impl ClockFormat {
    pub const fn key(self) -> &'static str {
        match self {
            Self::TwentyFour => "twenty-four",
            Self::Twelve => "twelve",
        }
    }

    pub fn of(key: &str) -> Self {
        match key {
            "twelve" => Self::Twelve,
            _ => Self::TwentyFour,
        }
    }

    /// `21:30`, or `9:30 PM`.
    pub fn time(self, hour: u32, minute: u32) -> String {
        match self {
            Self::TwentyFour => format!("{hour:02}:{minute:02}"),
            Self::Twelve => {
                let suffix = if hour < 12 { "AM" } else { "PM" };
                let shown = match hour % 12 {
                    0 => 12,
                    other => other,
                };
                format!("{shown}:{minute:02} {suffix}")
            }
        }
    }
}

fn read() -> ClockFormat {
    // Through the platform seam, not `window.local_storage()`: this value is read while the
    // tree renders, and a render that reaches for the document cannot run anywhere but a
    // browser. `platform::storage` answers `None` instead of failing.
    crate::platform::storage::get(KEY)
        .map_or(ClockFormat::TwentyFour, |value| ClockFormat::of(&value))
}

fn write(format: ClockFormat) {
    crate::platform::storage::set(KEY, format.key());
}

/// Creates the signal. Called before the first mount, like every app-lifetime signal.
pub fn install() {
    FORMAT.with_borrow_mut(|slot| {
        if slot.is_some() {
            return;
        }
        *slot = Some(RwSignal::new(read()));
    });
}

fn current() -> Option<RwSignal<ClockFormat>> {
    FORMAT.with_borrow(|slot| *slot)
}

/// The reader's clock format; 24-hour until they choose otherwise.
pub fn format() -> ClockFormat {
    current().map_or(ClockFormat::TwentyFour, |signal| signal.get_untracked())
}

/// The signal, for a control that shows the current choice.
pub fn signal() -> Option<RwSignal<ClockFormat>> {
    current()
}

/// Stores a choice and repaints every reader of it.
pub fn set(format: ClockFormat) {
    write(format);
    if let Some(signal) = current() {
        let _ = signal.try_update(|value| *value = format);
    }
}
