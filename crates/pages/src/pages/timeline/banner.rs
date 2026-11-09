//! The announcement a timeline shows above its posts, and the line a failed read adds.
//!
//! The instance decides which announcement is current; this component only draws the one it
//! is handed. The text arrives already reduced to what may be rendered, because sanitizing
//! the instance's HTML is the caller's job.

use leptos::prelude::*;

/// One announcement, ready to paint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Announcement {
    pub id: String,
    pub text: String,
}

/// The banner above a timeline, plus the small error line a failed announcement read adds.
///
/// With no `on_dismiss` the button keeps its place and is disabled: the server paints the
/// same banner, it just has nowhere to send the dismissal.
#[component]
pub fn TimelineBanner(
    #[prop(into)] current: Signal<Option<Announcement>>,
    #[prop(into)] error: Signal<Option<String>>,
    #[prop(optional_no_strip)] on_dismiss: Option<Callback<String>>,
) -> impl IntoView {
    view! {
        <Show when=move || current.get().is_some()>
            {move || {
                current
                    .get()
                    .map(|announcement| {
                        let id = announcement.id;
                        view! {
                            <div class="alert alert-info flex items-start gap-2 py-2 text-sm">
                                <span class="min-w-0 flex-1">{announcement.text}</span>
                                <button
                                    class="btn btn-ghost btn-xs"
                                    disabled=on_dismiss.is_none()
                                    on:click=move |_| {
                                        if let Some(on_dismiss) = on_dismiss {
                                            on_dismiss.run(id.clone());
                                        }
                                    }
                                >
                                    "Dismiss"
                                </button>
                            </div>
                        }
                    })
            }}
        </Show>
        <Show when=move || error.get().is_some()>
            <p class="text-xs text-error">{move || error.get().unwrap_or_default()}</p>
        </Show>
    }
}
