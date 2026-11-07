//! The code of conduct, or the sentence saying there is none.

use leptos::prelude::*;
use nodus_instance::model::InstanceInfo;

/// The rules, or a plain statement that the server publishes none.
#[component]
pub fn Rules(info: ReadSignal<Option<InstanceInfo>>) -> impl IntoView {
    let rules = move || info.get().map_or_else(Vec::new, |info| info.rules);
    let empty = move || rules().is_empty();

    view! {
        <Show when=move || !empty()>
            <div>
                <h3 class="mb-1 text-sm font-medium">"Rules"</h3>
                <ol class="list-decimal pl-5 text-sm">
                    <For each=rules key=|rule| rule.id.clone() let:rule>
                        <li class="py-0.5">
                            <span>{rule.text.clone()}</span>
                            {rule
                                .hint
                                .clone()
                                .map(|hint| view! { <span class="block text-xs opacity-60">{hint}</span> })}
                        </li>
                    </For>
                </ol>
            </div>
        </Show>

        <Show when=empty>
            <p class="text-sm opacity-60">"This instance publishes no rules."</p>
        </Show>
    }
}
