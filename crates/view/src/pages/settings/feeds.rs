//! Which timelines the navigation offers, as a list of switches.
//!
//! The choice is a browser preference owned by the caller; this card only reports the
//! intended value and whether the row can move at all.

use leptos::prelude::*;

use crate::components::panel::Panel;

/// One feed row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeedItem {
    pub key: String,
    pub label: String,
    pub description: String,
    pub enabled: bool,
    pub fixed: bool,
}

/// The list of feeds with a switch each.
#[component]
pub fn FeedsCard(
    #[prop(into)] feeds: Signal<Vec<FeedItem>>,
    #[prop(optional_no_strip)] on_toggle: Option<Callback<(String, bool)>>,
) -> impl IntoView {
    view! {
        <Panel title="Timelines">
            <p class="text-sm opacity-70">
                "Which timelines the navigation offers. Hiding one does not delete anything: turn it back on and it is where it was."
            </p>

            <div class="mt-1 flex flex-col gap-1">
                {move || {
                    feeds
                        .get()
                        .into_iter()
                        .map(|feed| {
                            let key = feed.key;
                            let fixed = feed.fixed;
                            let enabled = feed.enabled;
                            view! {
                                <label class="flex cursor-pointer items-start gap-3 rounded-box p-2 hover:bg-base-200">
                                    <input
                                        type="checkbox"
                                        class="toggle toggle-sm mt-0.5"
                                        prop:checked=enabled
                                        prop:disabled=fixed || on_toggle.is_none()
                                        on:change=move |event| {
                                            if let Some(on_toggle) = on_toggle {
                                                on_toggle.run((key.clone(), event_target_checked(&event)));
                                            }
                                        }
                                    />
                                    <span class="flex flex-col">
                                        <span class="text-sm font-medium">{feed.label}</span>
                                        <span class="text-xs opacity-60">{feed.description}</span>
                                    </span>
                                </label>
                            }
                        })
                        .collect_view()
                }}
            </div>

            <p class="mt-2 text-xs opacity-60">
                "Home stays: it is where a sign-in lands, and the timeline every client has. The Feed is this client's own recommendation screen, and it can be closed like the others."
            </p>
        </Panel>
    }
}
