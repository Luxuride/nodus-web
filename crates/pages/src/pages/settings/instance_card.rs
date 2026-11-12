//! The whole "This instance" card, so the caller only reads and reports.

use leptos::prelude::*;
use nodus_client_core::model::InstanceInfo;

use crate::components::panel::Panel;

use super::instance::{Identity, Rules};

/// What the server says about itself, or that it did not answer.
#[component]
pub fn InstanceCard(
    #[prop(into)] info: Signal<Option<InstanceInfo>>,
    #[prop(into)] failed: Signal<bool>,
) -> impl IntoView {
    view! {
        <Panel title="This instance">
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
        </Panel>
    }
}
