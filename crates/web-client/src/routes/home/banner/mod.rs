//! Announcement banner for the timeline screens.
//!
//! Only the first unread announcement is shown, and only until it is dismissed. The
//! instance sends these to say something operational ("maintenance tonight", "rules
//! updated"), so the banner states the content and gets out of the way; it does not
//! interrupt, and dismissing it calls the API so the next client agrees.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_client_core::model::Announcement;

use crate::routes::feed::posts::plain_text;
use crate::state::alive::Alive;
use crate::state::bridge::notices;

/// The announcement the instance wants read.
#[component]
pub fn Banner() -> impl IntoView {
    let (current, set_current) = signal(None::<Announcement>);
    let (error, set_error) = signal(None::<String>);
    let alive = Alive::new();

    // Loaded once per visit: an announcement is not a timeline and does not need a
    // cursor, so there is nothing to refresh.
    Effect::new(move |_| {
        let alive = alive;
        spawn_local(async move {
            match notices::announcements().await {
                Ok(list) => {
                    let now = String::from(js_sys::Date::new_0().to_iso_string());
                    let first = list
                        .into_iter()
                        .find(|announcement| announcement.should_show(&now));
                    alive.if_alive(|| set_current.set(first));
                }
                // A failed announcement is not worth an error banner of its own: the
                // page it sits on is already showing whether the timeline loaded.
                Err(message) => alive.if_alive(|| set_error.set(Some(message))),
            }
        });
    });

    let dismiss = move |id: String| {
        spawn_local(async move {
            alive.if_alive(|| set_current.set(None));
            if let Err(message) = notices::dismiss_announcement(&id).await {
                alive.if_alive(|| set_error.set(Some(message)));
            }
        });
    };

    view! {
        <nodus_pages::pages::timeline::banner::TimelineBanner
            current=Signal::derive(move || {
                current
                    .get()
                    .map(|announcement| nodus_pages::pages::timeline::banner::Announcement {
                        id: announcement.id,
                        text: plain_text(&announcement.content),
                    })
            })
            error=Signal::derive(move || error.get())
            on_dismiss=Some(Callback::new(dismiss))
        />
    }
}
