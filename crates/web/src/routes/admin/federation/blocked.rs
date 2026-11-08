//! Domains the instance refuses.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::{DomainBlock, DomainSeverity};

use crate::state::alive::Alive;
use crate::state::bridge::admin;

/// Blocked domains, with an add form.
#[component]
pub(super) fn BlockedList() -> impl IntoView {
    let alive = Alive::new();
    let (blocks, set_blocks) = signal(Vec::<DomainBlock>::new());
    let (domain, set_domain) = signal(String::new());
    let (suspend, set_suspend) = signal(true);
    let (busy, set_busy) = signal(true);
    let (error, set_error) = signal(None::<String>);

    let reload = Callback::new(move |()| {
        spawn_local(async move {
            let outcome = admin::domain_blocks().await;
            alive.if_alive(|| {
                set_busy.set(true);
                match outcome {
                    Ok(list) => {
                        set_blocks.set(list);
                        set_error.set(None);
                    }
                    Err(message) => set_error.set(Some(message)),
                }
                set_busy.set(false);
            });
        });
    });

    Effect::new(move |_| reload.run(()));

    let add = Callback::new(move |()| {
        let name = domain.get_untracked().trim().to_lowercase();
        if name.is_empty() {
            set_error.set(Some("Name the domain to block.".to_owned()));
            return;
        }
        let severity = if suspend.get_untracked() {
            DomainSeverity::Suspend
        } else {
            DomainSeverity::Silence
        };
        spawn_local(async move {
            match admin::create_domain_block(&name, severity).await {
                Ok(_) => {
                    alive.if_alive(|| set_domain.set(String::new()));
                    reload.run(());
                }
                Err(message) => alive.if_alive(|| set_error.set(Some(message))),
            }
        });
    });

    view! {
        <Show when=move || error.get().is_some()>
            <p class="text-sm text-error">{move || error.get().unwrap_or_default()}</p>
        </Show>

        <ul class="flex flex-col gap-2">
            <For each=move || blocks.get() key=|block| block.id.clone() let:block>
                {
                    let id = block.id.clone();
                    let label = format!("{} · {}", block.domain, block.severity.as_str());
                    view! {
                        <li class="flex items-center justify-between gap-2 rounded-box bg-base-100 px-3 py-2 shadow-sm">
                            <p class="truncate text-sm">{label}</p>
                            <button
                                class="btn btn-ghost btn-xs"
                                on:click=move |_| {
                                    let id = id.clone();
                                    spawn_local(async move {
                                        // A block that could not be removed is still
                                        // in force; the list is the record of that.
                                        if admin::delete_domain_block(&id).await.is_ok() {
                                            reload.run(());
                                        }
                                    });
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
                prop:value=domain
                on:input=move |event| set_domain.set(event_target_value(&event))
            />
            <label class="label cursor-pointer justify-start gap-2">
                <input
                    type="checkbox"
                    class="toggle toggle-sm"
                    prop:checked=suspend
                    on:change=move |event| set_suspend.set(event_target_checked(&event))
                />
                <span class="label-text">"Suspend entirely (off: silence from timelines)"</span>
            </label>
            <button
                class="btn btn-error btn-sm self-start"
                disabled=move || busy.get()
                on:click=move |_| add.run(())
            >
                "Block domain"
            </button>
        </div>
    }
}
