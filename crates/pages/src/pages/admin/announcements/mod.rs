//! Instance announcements, and the form that publishes them.

use leptos::prelude::*;
use nodus_client_core::model::Announcement;

use crate::components::panel::Panel;

/// Every announcement, with publishing controls.
#[component]
pub fn AnnouncementsPage(
    /// The announcements, as the caller last read them.
    #[prop(into)]
    items: Signal<Vec<Announcement>>,
    /// The message being written.
    #[prop(into)]
    draft: Signal<String>,
    /// Whether the new announcement is published immediately.
    #[prop(into)]
    published: Signal<bool>,
    /// Whether a read or write is already on its way.
    #[prop(into)]
    busy: Signal<bool>,
    /// The last read's or write's refusal.
    #[prop(into)]
    error: Signal<Option<String>>,
    /// Records the typed message.
    #[prop(optional)]
    on_draft: Option<Callback<String>>,
    /// Records the publish toggle.
    #[prop(optional)]
    on_published: Option<Callback<bool>>,
    /// Publishes the draft.
    #[prop(optional)]
    on_publish: Option<Callback<()>>,
    /// Withdraws one announcement.
    #[prop(optional)]
    on_delete: Option<Callback<String>>,
) -> impl IntoView {
    view! {
        <div class="flex flex-col gap-3">
            <Panel title="Announcements" description="Messages this instance shows to its members.">
                <Show when=move || error.get().is_some()>
                    <p class="alert alert-error alert-soft py-1 text-sm">
                        {move || error.get().unwrap_or_default()}
                    </p>
                </Show>
                <ul class="divide-y divide-base-300">
                    <For each=move || items.get() key=|announcement| announcement.id.clone() let:announcement>
                        {
                            let id = announcement.id.clone();
                            let target = StoredValue::new(id);
                            let status = if announcement.published { "published" } else { "draft" };
                            view! {
                                <li class="flex items-start justify-between gap-3 py-2">
                                    <div class="min-w-0">
                                        <p class="whitespace-pre-wrap text-sm">{announcement.content.clone()}</p>
                                        <p class="mt-1 text-xs opacity-60">
                                            {format!("{status} · {}", announcement.published_at.clone().unwrap_or_default())}
                                        </p>
                                    </div>
                                    <button
                                        class="btn btn-ghost btn-xs shrink-0"
                                        disabled=move || busy.get() || on_delete.is_none()
                                        on:click=move |_| {
                                            if let Some(on_delete) = on_delete {
                                                on_delete.run(target.get_value());
                                            }
                                        }
                                    >
                                        "Delete"
                                    </button>
                                </li>
                            }
                        }
                    </For>
                </ul>
                <Show when=move || items.get().is_empty()>
                    <p class="text-sm opacity-60">"No announcements yet."</p>
                </Show>
            </Panel>

            <Panel title="New announcement" description="Text is shown as written.">
                <form
                    class="flex flex-col gap-2"
                    on:submit=move |event| {
                        stop_submit(&event);
                        if let Some(on_publish) = on_publish {
                            on_publish.run(());
                        }
                    }
                >
                    <textarea
                        class="textarea textarea-bordered min-h-24"
                        placeholder="What should members know?"
                        prop:value=move || draft.get()
                        disabled=on_draft.is_none()
                        on:input=move |event| {
                            if let Some(on_draft) = on_draft {
                                on_draft.run(event_target_value(&event));
                            }
                        }
                    ></textarea>
                    <label class="label cursor-pointer justify-start gap-2">
                        <input
                            type="checkbox"
                            class="toggle toggle-sm"
                            prop:checked=move || published.get()
                            disabled=on_published.is_none()
                            on:change=move |event| {
                                if let Some(on_published) = on_published {
                                    on_published.run(event_target_checked(&event));
                                }
                            }
                        />
                        <span class="label-text">"Publish immediately"</span>
                    </label>
                    <button
                        class="btn btn-primary btn-sm self-start"
                        disabled=move || busy.get() || on_publish.is_none()
                    >
                        "Publish"
                    </button>
                </form>
            </Panel>
        </div>
    }
}

/// Keeps a submit from reloading the page on wasm, and keeps the native build's event used.
fn stop_submit(event: &leptos::ev::SubmitEvent) {
    #[cfg(not(target_arch = "wasm32"))]
    let _ = event;
    #[cfg(target_arch = "wasm32")]
    event.prevent_default();
}
