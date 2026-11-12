//! Domains this instance refuses.
//!
//! The outbound half of federation lives under Administration (`relays` and allowed
//! domains), because allowing a domain is a decision about the network, while blocking one is
//! a decision about what the people here have to see.
//!
//! The two fields of the add form are the caller's state and arrive as signals: a page may not
//! hold a draft. What it does own is the form's own layout, so the input, the severity toggle
//! and the press cannot drift apart from the list above them.

use leptos::prelude::*;
use nodus_client_core::model::DomainBlock;

/// Blocked domains, with an add form.
#[component]
pub fn FederationPage(
    /// The blocks this instance holds, as the caller last read them.
    #[prop(into)]
    blocks: Signal<Vec<DomainBlock>>,
    /// The domain being typed.
    #[prop(into)]
    domain: Signal<String>,
    /// Whether a new block refuses the domain entirely rather than silencing it.
    #[prop(into)]
    suspend: Signal<bool>,
    /// Whether a write is already on its way.
    #[prop(into)]
    busy: Signal<bool>,
    /// The last read's or write's refusal, when it had one.
    #[prop(into)]
    error: Signal<Option<String>>,
    /// Records the typed domain. Absent where there is nowhere to keep it.
    #[prop(optional)]
    on_domain: Option<Callback<String>>,
    /// Records the severity. Absent where nothing can be chosen.
    #[prop(optional)]
    on_suspend: Option<Callback<bool>>,
    /// Blocks the typed domain. Absent where nothing can be sent.
    #[prop(optional)]
    on_block: Option<Callback<()>>,
    /// Lifts one block. Absent where nothing can be sent.
    #[prop(optional)]
    on_remove: Option<Callback<String>>,
) -> impl IntoView {
    view! {
        <Show when=move || error.get().is_some()>
            <p class="text-sm text-error">{move || error.get().unwrap_or_default()}</p>
        </Show>

        <ul class="flex flex-col gap-2">
            <For each=move || blocks.get() key=|block| block.id.clone() let:block>
                {
                    let target = StoredValue::new(block.id.clone());
                    let label = format!("{} · {}", block.domain, block.severity.as_str());
                    view! {
                        <li class="flex items-center justify-between gap-2 rounded-box bg-base-100 px-3 py-2 shadow-sm">
                            <p class="truncate text-sm">{label}</p>
                            <button
                                class="btn btn-ghost btn-xs"
                                disabled=on_remove.is_none()
                                on:click=move |_| {
                                    // A block that could not be removed is still in force;
                                    // the list is the record of that.
                                    if let Some(on_remove) = on_remove {
                                        on_remove.run(target.get_value());
                                    }
                                }
                            >
                                "Remove"
                            </button>
                        </li>
                    }
                }
            </For>
        </ul>

        <div class="mt-4 flex flex-col gap-2">
            <input
                class="input input-bordered input-sm"
                placeholder="example.social"
                prop:value=move || domain.get()
                disabled=on_domain.is_none()
                on:input=move |event| {
                    if let Some(on_domain) = on_domain {
                        on_domain.run(event_target_value(&event));
                    }
                }
            />
            <label class="label cursor-pointer justify-start gap-2">
                <input
                    type="checkbox"
                    class="toggle toggle-sm"
                    prop:checked=move || suspend.get()
                    disabled=on_suspend.is_none()
                    on:change=move |event| {
                        if let Some(on_suspend) = on_suspend {
                            on_suspend.run(event_target_checked(&event));
                        }
                    }
                />
                <span class="label-text">"Suspend entirely (off: silence from timelines)"</span>
            </label>
            <button
                class="btn btn-error btn-sm self-start"
                disabled=move || busy.get() || on_block.is_none()
                on:click=move |_| {
                    if let Some(on_block) = on_block {
                        on_block.run(());
                    }
                }
            >
                "Block domain"
            </button>
        </div>
    }
}
