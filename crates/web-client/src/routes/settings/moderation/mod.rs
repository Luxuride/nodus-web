//! The mute and block lists, as shortcuts.
//!
//! These belong to the account rather than to moderation: muting and blocking are actions
//! every reader can take, so the links are here even in a build without the moderation
//! screens.

use leptos::prelude::*;
use nodus_pages::pages::settings::account::ModerationLinks as SharedModerationLinks;

use crate::router::{self, Route};

#[component]
pub(crate) fn ModerationLinks() -> impl IntoView {
    view! {
        <SharedModerationLinks
            on_mutes=Some(Callback::new(move |()| router::navigate(Route::Mutes)))
            on_blocks=Some(Callback::new(move |()| router::navigate(Route::Blocks)))
        />
    }
}
