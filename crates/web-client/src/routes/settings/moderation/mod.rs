//! The mute and block lists, as shortcuts.
//!
//! These belong to the account rather than to moderation: muting and blocking are actions
//! every reader can take, so the links are here even in a build without the moderation
//! screens.

use leptos::prelude::*;

use crate::router::{self, Route};

#[component]
pub(crate) fn ModerationLinks() -> impl IntoView {
    view! {
        <nodus_view::components::panel::Panel title="Moderation">
                <p class="text-sm opacity-70">
                    "Accounts you have muted or blocked stay hidden until you undo it here."
                </p>
                <div class="card-actions justify-start gap-2">
                    <a
                        class="btn btn-outline btn-sm"
                        href=Route::Mutes.path()
                        on:click=move |event| router::link(event, Route::Mutes)
                    >
                        "Muted accounts"
                    </a>
                    <a
                        class="btn btn-outline btn-sm"
                        href=Route::Blocks.path()
                        on:click=move |event| router::link(event, Route::Blocks)
                    >
                        "Blocked accounts"
                    </a>
                </div>
        </nodus_view::components::panel::Panel>
    }
}
