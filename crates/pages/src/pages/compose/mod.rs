//! The compose screens: writing a status, and editing one.
//!
//! Two addresses, one form: `/compose` starts empty, `/compose/{id}` reads the status first.
//! The form itself is [`form`]; what an edit shows while it waits is here, because that is
//! the one shape the two entry points do not share.

use leptos::prelude::*;

pub mod attachments;
pub mod form;
pub mod inline;

pub use form::ComposeForm;
pub use inline::InlineComposer;

/// What an edit shows while the status it edits is still being read.
///
/// The wait and the refusal are the same two lines the form paints an error with, so a
/// reader who arrives at `/compose/{id}` sees one vocabulary, not two.
#[component]
pub fn ComposeLoading(#[prop(into)] error: Signal<Option<String>>) -> impl IntoView {
    view! {
        <Show when=move || error.get().is_some()>
            <p class="text-error text-xs">{move || error.get().unwrap_or_default()}</p>
        </Show>
        <p class="text-sm opacity-60">"Loading the status…"</p>
    }
}
