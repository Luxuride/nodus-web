//! What this instance is announcing right now, read through the one endpoint that exists.
//!
//! The API has `GET /api/v1/announcements` (read) and `POST …/:id/dismiss`, and nothing
//! else: there is no administration endpoint for creating or withdrawing one at any version
//! the documentation covers, so `POST /api/v1/admin/announcements` answers the instance's
//! HTML 404 page. Publishing therefore is not offered here — the panel says so plainly
//! rather than forwarding the reader to a page this app cannot drive.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::Announcement;

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
            <nodus_view::components::panel::Panel title="Published now">
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
            </nodus_view::components::panel::Panel>

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
