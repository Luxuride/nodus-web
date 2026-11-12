//! What this instance is announcing right now, read through the one endpoint that exists.
//!
//! The API has `GET /api/v1/announcements` (read) and `POST …/:id/dismiss`, and nothing else:
//! there is no administration endpoint for creating or withdrawing one at any version the
//! documentation covers, so `POST /api/v1/admin/announcements` answers the instance's HTML
//! 404 page. Publishing therefore is not offered here — the panel says so plainly rather than
//! forwarding the reader to a page this app cannot drive.
//!
//! Nothing on this page is a verb, so its whole shape is markup: the list, its empty line, and
//! the panel that explains why there is no form.

use leptos::prelude::*;
use nodus_client_core::model::Announcement;

use crate::components::panel::Panel;

/// What is published now, and the honest statement about publishing.
#[component]
pub fn AnnouncementsPage(
    /// The announcements, as the caller last read them.
    #[prop(into)]
    items: Signal<Vec<Announcement>>,
    /// The last read's refusal, when it had one.
    #[prop(into)]
    error: Signal<Option<String>>,
) -> impl IntoView {
    let published = move || {
        items
            .get()
            .into_iter()
            .map(|announcement| {
                let ends = announcement.ends_at.clone();
                view! {
                    <li class="border-b border-base-300 py-2 last:border-0">
                        <p class="whitespace-pre-wrap text-sm">{announcement.content.clone()}</p>
                        <p class="mt-1 text-xs opacity-60">
                            {match ends {
                                Some(ends) => format!("Published until {ends}"),
                                None => "Published without an end date".to_owned(),
                            }}
                        </p>
                    </li>
                }
            })
            .collect::<Vec<_>>()
    };

    view! {
        <div class="flex flex-col gap-3">
            <Panel title="Published now">
                <Show
                    when=move || !items.get().is_empty()
                    fallback=move || {
                        view! {
                            <p class="text-sm opacity-70">
                                "This instance is not announcing anything."
                            </p>
                        }
                    }
                >
                    <ul>{published}</ul>
                </Show>
                {move || error.get().map(|message| view! {
                    <p class="alert alert-error alert-soft py-1 text-sm">{message}</p>
                })}
            </Panel>

            <div class="card border border-dashed border-base-300 bg-base-100">
                <div class="card-body gap-1 p-4">
                    <h2 class="text-sm font-semibold">"Publishing has no API"</h2>
                    <p class="text-sm opacity-70">
                        "Mastodon exposes no request for creating or withdrawing an
                         announcement, so no client can publish one. Server staff have to do
                         it in the instance's own administration pages."
                    </p>
                </div>
            </div>
        </div>
    }
}
