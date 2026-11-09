//! Which timelines the navigation offers.
//!
//! A preference, not a permission: hiding "Local" does not stop the client from being
//! able to read it, it stops the rail from offering it. Nothing here talks to the
//! instance — Mastodon has no API for "which timelines this person reads".

use leptos::prelude::*;

use crate::state::feeds::{self as prefs, Feed};

/// One switch per feed, each saying what it would hide.
#[component]
pub fn FeedsCard() -> impl IntoView {
    let Some(changed) = prefs::revision() else {
        return view! { <div class="text-sm opacity-70">"Preferences are not available."</div> }
            .into_any();
    };

    view! {
        <section class="card border border-base-300 bg-base-100">
            <div class="card-body p-4">
                <h2 class="card-title text-base">"Timelines"</h2>
                <p class="text-sm opacity-70">
                    "Which timelines the navigation offers. Hiding one does not delete anything: turn it back on and it is where it was."
                </p>

                <div class="mt-1 flex flex-col gap-1">
                    <For each=|| Feed::ALL key=|feed| feed.key() let:feed>
                        {
                            let on = move || {
                                changed.get();
                                prefs::enabled(feed)
                            };
                            // Home has no switch: it is the front door, and a toggle that
                            // cannot be turned off reads as a bug.
                            let fixed = !feed.optional();
                            view! {
                                <label class="flex cursor-pointer items-start gap-3 rounded-box p-2 hover:bg-base-200">
                                    <input
                                        type="checkbox"
                                        class="toggle toggle-sm mt-0.5"
                                        prop:checked=on
                                        prop:disabled=fixed
                                        on:change=move |event| {
                                            prefs::set(feed, event_target_checked(&event));
                                        }
                                    />
                                    <span class="flex flex-col">
                                        <span class="text-sm font-medium">{feed.label()}</span>
                                        <span class="text-xs opacity-60">{feed.description()}</span>
                                    </span>
                                </label>
                            }
                        }
                    </For>
                </div>

                <p class="mt-2 text-xs opacity-60">
                    "Home stays: it is where a sign-in lands, and the timeline every client
                     has. The Feed is this client's own recommendation screen, and it can be
                     closed like the others."
                </p>
            </div>
        </section>
    }
    .into_any()
}
