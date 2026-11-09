//! The submit row's shell: the label and the disabled rules live in the view; what Cancel
//! does does not, because leaving the composer means a route and routes are this crate's.
//!
//! [`ComposerActions`] is the same component the inline composer ends with — the only
//! difference between the two callers is the two verbs.

use leptos::prelude::*;
use nodus_view::pages::compose::form::buttons::ComposerActions as ActionsRow;

use crate::router::{self, Route};

/// Post or save, plus cancel.
#[component]
pub fn ComposerActions(
    text: ReadSignal<String>,
    busy: ReadSignal<bool>,
    editing: StoredValue<Option<String>>,
    on_submit: Callback<()>,
) -> impl IntoView {
    // The label asks whether this is an edit. That is fixed for the life of the form, and a
    // `StoredValue` is read, not tracked, so it is read once here rather than threaded into
    // the view as a signal.
    let editing = editing.get_value().is_some();

    view! {
        <ActionsRow
            text=text
            busy=busy
            editing=editing
            on_submit=on_submit
            on_cancel=Callback::new(move |()| router::navigate(Route::Home))
        />
    }
}
