//! The reader's choice about media behind a content warning.
//!
//! A setting, not a policy: the server never learns it and nothing about it is synced.
//! It is stored in `localStorage` because the alternative — asking on every post — is
//! exactly what a reader who has already decided does not want.

use leptos::prelude::*;

use super::super::status::sensitive;

/// One switch, applied to every sensitive post from now on.
#[component]
pub(super) fn SensitiveCard() -> impl IntoView {
    let (automatic, set_automatic) = signal(sensitive::reveals_automatically());

    let toggle = move |_| {
        let next = !automatic.get_untracked();
        sensitive::set_automatic(next);
        set_automatic.set(next);
    };

    view! {
        <div class="card bg-base-100 shadow-sm">
            <div class="card-body gap-2">
                <h2 class="card-title text-base">"Sensitive media"</h2>
                <label class="flex cursor-pointer items-center justify-between gap-4">
                    <span class="text-sm">
                        "Show sensitive media without asking"
                        <span class="block text-xs opacity-60">
                            "Otherwise every post marked sensitive waits for a click. "
                            "This preference stays in this browser."
                        </span>
                    </span>
                    <input
                        type="checkbox"
                        class="toggle toggle-sm"
                        prop:checked=move || automatic.get()
                        on:change=toggle
                    />
                </label>
            </div>
        </div>
    }
}
