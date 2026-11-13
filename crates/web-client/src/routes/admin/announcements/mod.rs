//! Announcements: staff read every one, publish and withdraw.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_client_core::model::{Announcement, AnnouncementDraft};
use nodus_pages::pages::admin::announcements::AnnouncementsPage;

use crate::state::alive::Alive;
use crate::state::bridge::admin;

#[component]
pub fn Announcements() -> impl IntoView {
    let (items, set_items) = signal(Vec::<Announcement>::new());
    let (draft, set_draft) = signal(String::new());
    let (published, set_published) = signal(true);
    let (busy, set_busy) = signal(false);
    let (error, set_error) = signal(None::<String>);

    let (tick, set_tick) = signal(0_u32);
    let reload = Callback::new(move |()| set_tick.update(|value| *value = value.wrapping_add(1)));

    Effect::new(move |_| {
        tick.get();
        let alive = Alive::new();
        set_busy.set(true);
        spawn_local(async move {
            match admin::admin_announcements().await {
                Ok(found) => alive.if_alive(|| {
                    set_items.set(found);
                    set_error.set(None);
                }),
                Err(message) => alive.if_alive(|| set_error.set(Some(message))),
            }
            alive.if_alive(|| set_busy.set(false));
        });
    });

    let publish = Callback::new(move |()| {
        let text = draft.get().trim().to_owned();
        if text.is_empty() {
            set_error.set(Some("Write something first.".to_owned()));
            return;
        }
        let mut entry = AnnouncementDraft::new(text);
        entry.published = published.get();
        let alive = Alive::new();
        set_busy.set(true);
        spawn_local(async move {
            match admin::create_announcement(entry).await {
                Ok(_) => alive.if_alive(|| {
                    set_draft.set(String::new());
                    set_error.set(None);
                    reload.run(());
                }),
                Err(message) => alive.if_alive(|| set_error.set(Some(message))),
            }
            alive.if_alive(|| set_busy.set(false));
        });
    });

    let delete = Callback::new(move |id: String| {
        let alive = Alive::new();
        set_busy.set(true);
        spawn_local(async move {
            match admin::delete_announcement(&id).await {
                Ok(()) => alive.if_alive(|| {
                    set_error.set(None);
                    reload.run(());
                }),
                Err(message) => alive.if_alive(|| set_error.set(Some(message))),
            }
            alive.if_alive(|| set_busy.set(false));
        });
    });

    view! {
        <AnnouncementsPage
            items=items
            draft=draft
            published=published
            busy=busy
            error=error
            on_draft=Some(Callback::new(move |value: String| set_draft.set(value)))
            on_published=Some(Callback::new(move |value: bool| set_published.set(value)))
            on_publish=Some(publish)
            on_delete=Some(delete)
        />
    }
}
