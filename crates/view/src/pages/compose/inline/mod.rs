//! The composer that sits above the home timeline.
//!
//! Always open, never toggled: writing is the reason the home timeline exists, and a
//! button that expands a form is one decision more than a box that is simply there. It
//! is deliberately the *short* path — text, media, visibility, and a link to the full
//! composer for anything else (scheduling, a content warning, a reply's thread context).
//!
//! The card, its heading and the three lines that report what happened are the same on
//! every screen; the writer and the controls under it are slots, because they own the
//! editor, the upload and the clock.

use leptos::prelude::*;

pub mod bar;
pub mod drafts_panel;
pub mod emoji;
pub mod emoji_toggle;
pub mod hints;
pub mod poll;
pub mod preview;
pub mod schedule;

pub use bar::ComposerBar;
pub use drafts_panel::{Draft, DraftsPanel};
pub use emoji::Suggestions;
pub use preview::ComposerPreview;

/// What a published post says afterwards: the visibility it went out with, and the
/// language, named the way the composer names it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Posted {
    /// The visibility's own name, `Visibility::as_str`.
    pub visibility: String,
    /// The language's name, or "instance default" when none was chosen.
    pub language: String,
}

/// A box above the timeline: write, attach, post.
#[component]
pub fn InlineComposer(
    /// The suggestions, the drafts popup and the editor.
    body: ChildrenFn,
    /// The bar, the poll and the attachment list, in that order.
    controls: ChildrenFn,
    /// The last send's refusal.
    #[prop(optional)]
    error: Option<Signal<Option<String>>>,
    /// The line a queued post leaves behind.
    #[prop(optional)]
    notice: Option<Signal<Option<String>>>,
    /// What a published post went out as.
    #[prop(optional)]
    done: Option<Signal<Option<Posted>>>,
) -> impl IntoView {
    let body = StoredValue::new(body);
    let controls = StoredValue::new(controls);

    view! {
        <crate::ComposerCard>
            <h2 class="text-xs font-semibold uppercase tracking-wide opacity-50">"New post"</h2>
            {body.with_value(|body| body())}
            {controls.with_value(|controls| controls())}

            {move || error.and_then(|error| error.get()).map(|message| view! {
                <p class="text-error text-xs" role="alert">{message}</p>
            })}

            {move || notice.and_then(|notice| notice.get()).map(|line| view! {
                <p class="text-xs text-success">{line}</p>
            })}

            {move || done.and_then(|done| done.get()).map(|posted| view! {
                <p class="flex flex-wrap items-center gap-1 text-xs text-success" role="status">
                    "Post published as"
                    <span class="inline-flex items-center rounded-lg border border-base-300 bg-base-100 px-1.5 py-0.5 text-base-content">{posted.visibility}</span>
                    <span class="inline-flex items-center rounded-lg border border-base-300 bg-base-100 px-1.5 py-0.5 text-base-content">{posted.language}</span>
                </p>
            })}
        </crate::ComposerCard>
    }
}
