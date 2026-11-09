//! The instance behind this account: what it calls itself, what it asks of you, who runs it.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::InstanceInfo;

use crate::state::alive::Alive;
use crate::state::bridge::actions;

mod identity;
mod rules;

use identity::Identity;
use rules::Rules;

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
        <nodus_view::components::panel::Panel title="This instance">

                <Show when=move || failed.get()>
                    <p class="text-sm text-error">
                        "This server did not answer with its details. That is a fact about the server or the connection, not about your account."
                    </p>
                </Show>

                <Show when=move || info.get().is_none() && !failed.get()>
                    <p class="text-sm opacity-60">"Asking the server who it is…"</p>
                </Show>

                <Show when=move || info.get().is_some()>
                    <Identity info=info/>
                    <Rules info=info/>
                </Show>
        </nodus_view::components::panel::Panel>
    }
}
