//! Announcements, which Mastodon does not let a client write.
//!
//! The API has `GET /api/v1/announcements` (read) and `POST …/:id/dismiss`, and nothing
//! else: there is no administration endpoint for creating or withdrawing one at any
//! version the documentation covers, so `POST /api/v1/admin/announcements` answers the
//! instance's HTML 404 page. This panel says so and links to the page that can.

use leptos::prelude::*;

use crate::state::bridge;

#[component]
pub fn Announcements() -> impl IntoView {
    let page = move || {
        bridge::with_sdk(|sdk| sdk.state().session.get())
            .map(|session| format!("https://{}/admin/announcements", session.host))
    };

    view! {
        <div class="prose prose-sm max-w-none">
            <h2 class="card-title text-base">"Announcements"</h2>
            <p class="text-sm opacity-70">
                "Mastodon has no API for writing announcements: a client can read them (which
                 is how the banner on the home screen works) and dismiss them, but creating or
                 withdrawing one happens on the instance's own administration page. Open it
                 below."
            </p>
            {move || page().map(|url| view! {
                <a
                    class="btn btn-outline btn-sm w-fit"
                    href=url
                    target="_blank"
                    rel="noopener noreferrer"
                >
                    "Open announcements on this instance"
                </a>
            })}
        </div>
    }
}
