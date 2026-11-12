//! Administration: the instance itself — what it counted, who it talks to, what it says.
//!
//! Distinct from Moderation on purpose. Moderating is a decision about people (reports,
//! accounts, approvals, blocked domains); administering is a decision about the server (the
//! dashboard, relays, allowed domains, announcements). Mastodon draws the same line, and a
//! moderator who is not an administrator should not have to read server levers to find the
//! report queue.
//!
//! The area — heading, staff gate and section strip — is drawn in `nodus-view`. What stays
//! here is the staff probe and which section's body to hand the page.

use leptos::prelude::*;
use leptos::task::spawn_local;

use nodus_pages::pages::admin::{AdminPage, Section};
use nodus_ui::chrome::StaffStanding;

use crate::router::{self, Route};
use crate::state::alive::Alive;
use crate::state::bridge::admin;

use self::announcements::Announcements;
use self::domains::Domains;
use self::overview::Overview;

mod announcements;
mod domains;
mod overview;

/// The administration area for one section.
#[expect(
    clippy::needless_pass_by_value,
    reason = "a Leptos component receives its props by value"
)]
#[component]
pub fn Admin(section: String) -> impl IntoView {
    let section = StoredValue::new(Section::of(&section));
    let alive = Alive::new();
    // `None` is "not asked yet", `Some(NeedsGrant)` is "this token lacks the scope", and the
    // gate answers each of those differently: a spinner, a grant prompt, or the screens.
    let (allowed, set_allowed) = signal(None::<admin::Staff>);
    let probe = {
        move || {
            spawn_local(async move {
                let verdict = admin::staff().await;
                alive.if_alive(|| set_allowed.set(Some(verdict)));
            });
        }
    };
    Effect::new(move |_| probe());
    let retry = Callback::new(move |()| {
        set_allowed.set(None);
        probe();
    });
    // The grant itself is browser-only — it leaves for the instance's own authorize page —
    // so it lives here and reaches the gate as a verb.
    let grant = Callback::new(move |()| {
        spawn_local(async move {
            // The grant navigates away to the instance; the retry covers the case where the
            // reader comes back without finishing it.
            let _ = crate::routes::settings::admin::request_grant().await;
            retry.run(());
        });
    });

    // The page speaks the shared standing; the probe's answer is translated once here.
    let standing = Signal::derive(move || allowed.get().map(standing_of));

    view! {
        <AdminPage
            active=section.get_value().slug()
            standing=standing
            on_grant=Some(grant)
            on_navigate=Some(Callback::new(|slug: String| router::navigate(Route::Admin(slug))))
        >
            {move || match section.get_value() {
                Section::Overview => view! { <Overview/> }.into_any(),
                Section::Domains => view! { <Domains/> }.into_any(),
                Section::Announcements => view! { <Announcements/> }.into_any(),
            }}
        </AdminPage>
    }
}

/// The bridge's verdict, in the gate's words.
fn standing_of(staff: admin::Staff) -> StaffStanding {
    match staff {
        admin::Staff::Yes => StaffStanding::Yes,
        admin::Staff::No => StaffStanding::No,
        admin::Staff::NeedsGrant => StaffStanding::NeedsGrant,
    }
}
