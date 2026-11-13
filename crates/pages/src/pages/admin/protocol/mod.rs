//! What this server speaks, as the SDK detects it.

use leptos::prelude::*;

use crate::components::panel::Panel;

/// Detected protocol and Nodus feature names.
#[component]
pub fn ProtocolPage(
    /// Human-readable protocol name, when detection succeeded.
    #[prop(into)]
    protocol: Signal<Option<String>>,
    /// Nodus capabilities, when the server advertises them.
    #[prop(into)]
    features: Signal<Vec<String>>,
    /// Why detection did not answer.
    #[prop(into)]
    error: Signal<Option<String>>,
) -> impl IntoView {
    view! {
        <Panel
            title="Protocol"
            description="The richest client API this server publishes."
        >
            <Show when=move || error.get().is_some()>
                <p class="text-sm text-error">{move || error.get().unwrap_or_default()}</p>
            </Show>
            <p class="text-sm">
                "Detected: "
                <strong>{move || protocol.get().unwrap_or_else(|| "not detected".to_owned())}</strong>
            </p>
            <Show when=move || !features.get().is_empty()>
                <div class="mt-3">
                    <p class="text-xs uppercase tracking-wide opacity-60">"Nodus features"</p>
                    <div class="mt-1 flex flex-wrap gap-1">
                        <For each=move || features.get() key=|feature| feature.clone() let:feature>
                            <span class="badge badge-outline badge-sm">{feature}</span>
                        </For>
                    </div>
                </div>
            </Show>
            <p class="mt-3 text-xs opacity-60">
                "A protocol missing here is unsupported, not broken: the SDK's typed surfaces answer with an error instead of silently doing nothing."
            </p>
        </Panel>
    }
}
