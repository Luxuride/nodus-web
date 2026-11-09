//! The reader's choice about media behind a content warning.
//!
//! A setting, not a policy: the server never learns it and nothing about it is synced.
//! It is stored in `localStorage` because the alternative — asking on every post — is
//! exactly what a reader who has already decided does not want.

use leptos::prelude::*;
use nodus_pages::pages::settings::profile::SensitiveCard as SharedSensitiveCard;

use super::super::status::sensitive;

/// One switch, applied to every sensitive post from now on.
#[component]
pub(super) fn SensitiveCard() -> impl IntoView {
    let (automatic, set_automatic) = signal(sensitive::reveals_automatically());

    view! {
        <SharedSensitiveCard
            automatic=Signal::derive(move || automatic.get())
            on_toggle=Some(Callback::new(move |next: bool| {
                sensitive::set_automatic(next);
                set_automatic.set(next);
            }))
        />
    }
}
