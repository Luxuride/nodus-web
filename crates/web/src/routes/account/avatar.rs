//! The profile picture, clickable to be seen properly.
//!
//! Its own component so the profile card stays under the file-length limit the
//! conventions ask for, and because "a picture expands" is one behaviour, not markup the
//! card should carry.

use leptos::prelude::*;

/// The account's avatar; clicking it opens the app's one image expander.
#[component]
pub(super) fn Avatar(src: ReadSignal<String>) -> impl IntoView {
    view! {
        // The picture expands rather than navigates: the reader is already on this
        // profile, and a photograph is something to look at, not a second copy of the
        // page they are standing on.
        <button
            class="avatar cursor-zoom-in"
            type="button"
            title="Expand picture"
            on:click=move |_| {
                let url = src.get_untracked();
                if !url.is_empty() {
                    crate::media::expand_one(url, String::new());
                }
            }
        >
            <div class="w-16 rounded-full bg-base-300">
                <img width="64" height="64" decoding="async" src=src alt="" referrerpolicy="no-referrer"/>
            </div>
        </button>
    }
}
