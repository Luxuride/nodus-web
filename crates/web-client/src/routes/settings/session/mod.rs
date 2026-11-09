//! Who this browser is signed in as, and the way out.

use leptos::prelude::*;
use nodus_pages::pages::settings::account::SessionCard as SharedSessionCard;

use crate::state::bridge::sign_out;

/// The current session, plus the one control that always applies to it.
#[component]
pub(crate) fn SessionCard(
    host: impl Fn() -> String + Copy + Send + Sync + 'static,
    acct: impl Fn() -> String + Copy + Send + Sync + 'static,
) -> impl IntoView {
    view! {
        <SharedSessionCard
            host=Signal::derive(host)
            acct=Signal::derive(acct)
            on_sign_out=Some(Callback::new(move |()| sign_out()))
        />
    }
}
