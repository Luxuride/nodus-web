//! Invites, which Mastodon does not hand to clients.
//!
//! `GET /api/v1/invites` answers 404 on every Mastodon we checked (4.3.8 on this
//! instance, plus mastodon.social, fosstodon.org and chaos.social): invites are served
//! only to the server's own web UI, and creating one needs that UI's session cookie and
//! CSRF token. A client cannot mint them, so this card says where they live instead of
//! offering a button that always fails.

use leptos::prelude::*;

use crate::state::bridge;

#[component]
pub(super) fn Invites() -> impl IntoView {
    let page = move || {
        bridge::with_sdk(|sdk| sdk.state().session.get())
            .map(|session| format!("https://{}/admin/invites", session.host))
    };

    view! {
        <div class="card mb-4 border border-base-300 bg-base-100 shadow-sm">
            <div class="card-body gap-2 p-4">
                <h2 class="card-title text-base">"Invites"</h2>
                <p class="text-sm opacity-70">
                    "Mastodon keeps invites in its own administration pages: the API does not
                     expose them, so no third-party client can create or list them. Open them
                     on this instance and copy the link you want to share."
                </p>
                {move || page().map(|url| view! {
                    <a
                        class="btn btn-outline btn-sm w-fit"
                        href=url
                        target="_blank"
                        rel="noopener noreferrer"
                    >
                        "Open invite settings on this instance"
                    </a>
                })}
            </div>
        </div>
    }
}
