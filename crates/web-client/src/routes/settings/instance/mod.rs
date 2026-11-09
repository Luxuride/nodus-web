//! The instance behind this account: what it calls itself, what it asks of you, who runs it.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::InstanceInfo;
use nodus_view::pages::settings::instance_card::InstanceCard as SharedInstanceCard;

use crate::state::alive::Alive;
use crate::state::bridge::actions;

/// Reads `/api/v2/instance` and states it plainly.
///
/// This card is never empty: a server that publishes no rules still has a name, a version
/// and a contact address, and an empty tab is indistinguishable from a broken one.
#[component]
pub fn InstanceCard() -> impl IntoView {
    let (info, set_info) = signal(None::<InstanceInfo>);
    let (failed, set_failed) = signal(false);
    let alive = Alive::new();

    Effect::new(move |_| {
        spawn_local(async move {
            match actions::instance_info().await {
                Ok(payload) => alive.if_alive(|| set_info.set(Some(payload))),
                Err(_) => alive.if_alive(|| set_failed.set(true)),
            }
        });
    });

    view! {
        <SharedInstanceCard
            info=Signal::derive(move || info.get())
            failed=Signal::derive(move || failed.get())
        />
    }
}
