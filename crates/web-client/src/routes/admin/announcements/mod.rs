//! What this instance is announcing right now, read through the one endpoint that exists.
//!
//! The API has `GET /api/v1/announcements` (read) and `POST …/:id/dismiss`, and nothing
//! else: there is no administration endpoint for creating or withdrawing one at any version
//! the documentation covers, so `POST /api/v1/admin/announcements` answers the instance's
//! HTML 404 page. Publishing therefore is not offered here — the panel says so plainly
//! rather than forwarding the reader to a page this app cannot drive.
//!
//! The panels are drawn in `nodus-view`; what stays here is the read.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_client_core::model::Announcement;
use nodus_pages::pages::admin::announcements::AnnouncementsPage;

use crate::state::alive::Alive;
use crate::state::bridge::notices::announcements;

#[component]
pub fn Announcements() -> impl IntoView {
    let (items, set_items) = signal(Vec::<Announcement>::new());
    let (error, set_error) = signal(None::<String>);

    Effect::new(move |_| {
        let alive = Alive::new();
        spawn_local(async move {
            match announcements().await {
                Ok(found) => alive.if_alive(|| set_items.set(found)),
                Err(message) => alive.if_alive(|| set_error.set(Some(message))),
            }
        });
    });

    view! {
        <AnnouncementsPage items=items error=error/>
    }
}
