//! Two preferences about how posts are shown: the clock and sensitive media.
//!
//! Both are stored by the caller, so the controls here report a choice and never read or
//! write a browser.

use leptos::prelude::*;

use crate::components::panel::Panel;

/// How a clock time is written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockStyle {
    TwentyFour,
    Twelve,
}

/// How this reader writes a clock time.
#[component]
pub fn ClockCard(
    #[prop(into)] selected: Signal<ClockStyle>,
    #[prop(into)] sample: Signal<String>,
    #[prop(optional_no_strip)] on_select: Option<Callback<ClockStyle>>,
) -> impl IntoView {
    let pick = move |style: ClockStyle| {
        if let Some(on_select) = on_select {
            on_select.run(style);
        }
    };

    view! {
        <Panel
            title="Clock"
            description="Times in the composer and the schedule panel follow this. Nothing is sent to the instance either way."
        >
            <div class="flex gap-2">
                <button
                    type="button"
                    class="btn btn-sm"
                    class:btn-primary=move || selected.get() == ClockStyle::TwentyFour
                    class:btn-outline=move || selected.get() != ClockStyle::TwentyFour
                    disabled=on_select.is_none()
                    on:click=move |_| pick(ClockStyle::TwentyFour)
                >
                    "24-hour"
                </button>
                <button
                    type="button"
                    class="btn btn-sm"
                    class:btn-primary=move || selected.get() == ClockStyle::Twelve
                    class:btn-outline=move || selected.get() != ClockStyle::Twelve
                    disabled=on_select.is_none()
                    on:click=move |_| pick(ClockStyle::Twelve)
                >
                    "AM / PM"
                </button>
                <span class="ml-auto self-center text-xs opacity-60 tabular-nums">
                    {move || sample.get()}
                </span>
            </div>
        </Panel>
    }
}

/// One switch, applied to every sensitive post from now on.
#[component]
pub fn SensitiveCard(
    #[prop(into)] automatic: Signal<bool>,
    #[prop(optional_no_strip)] on_toggle: Option<Callback<bool>>,
) -> impl IntoView {
    view! {
        <Panel title="Sensitive media">
            <label class="flex cursor-pointer items-center justify-between gap-4">
                <span class="text-sm">
                    "Show sensitive media without asking"
                    <span class="block text-xs opacity-60">
                        "Otherwise every post marked sensitive waits for a click. This preference stays in this browser."
                    </span>
                </span>
                <input
                    type="checkbox"
                    class="toggle toggle-sm"
                    prop:checked=move || automatic.get()
                    disabled=on_toggle.is_none()
                    on:change=move |event| {
                        if let Some(on_toggle) = on_toggle {
                            on_toggle.run(event_target_checked(&event));
                        }
                    }
                />
            </label>
        </Panel>
    }
}
