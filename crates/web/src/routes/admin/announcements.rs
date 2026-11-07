//! Instance announcements: what every account sees on the home screen.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::Announcement;

use crate::state::alive::Alive;
use crate::state::bridge::{admin, revision};

/// The published announcements, and a form that adds one.
#[component]
pub fn Announcements() -> impl IntoView {
    let revision = revision();
    let alive = Alive::new();
    let (items, set_items) = signal(Vec::<Announcement>::new());
    let (content, set_content) = signal(String::new());
    let (ends_at, set_ends_at) = signal(String::new());
    let (busy, set_busy) = signal(false);
    let (error, set_error) = signal(None::<String>);

    let load = move || {
        let alive = Alive::new();
        spawn_local(async move {
            match admin::announcements().await {
                Ok(list) => alive.if_alive(|| set_items.set(list)),
                Err(message) => alive.if_alive(|| set_error.set(Some(message))),
            }
        });
    };

    Effect::new(move |_| {
        revision.get();
        load();
    });

    let publish = move |()| {
        let text = content.get_untracked().trim().to_owned();
        if text.is_empty() {
            set_error.set(Some("An announcement needs something to say.".to_owned()));
            return;
        }
        // A blank end date means "no end", not "the epoch": the API reads an absent
        // `ends_at` as an announcement that stays up until it is withdrawn.
        let ends = ends_at.get_untracked();
        let ends = (!ends.trim().is_empty()).then(|| ends.trim().to_owned());
        set_busy.set(true);
        set_error.set(None);

        spawn_local(async move {
            match admin::create_announcement(&text, ends.as_deref()).await {
                Ok(_) => alive.if_alive(|| {
                    set_content.set(String::new());
                    set_ends_at.set(String::new());
                    set_busy.set(false);
                }),
                Err(message) => alive.if_alive(|| {
                    set_error.set(Some(message));
                    set_busy.set(false);
                }),
            }
            alive.if_alive(load);
        });
    };

    view! {
        <div class="flex flex-col gap-3">
            <div class="card border border-base-300 bg-base-100 shadow-sm">
                <div class="card-body gap-2 p-4">
                    <h2 class="card-title text-base">"Publish an announcement"</h2>
                    <textarea
                        class="textarea textarea-bordered min-h-24 w-full text-sm"
                        placeholder="Maintenance on Sunday, rules update, a call for moderators…"
                        prop:value=content
                        on:input=move |event| set_content.set(event_target_value(&event))
                    ></textarea>
                    <div class="flex flex-wrap items-center gap-2">
                        <label class="flex items-center gap-2 text-xs opacity-70">
                            "Ends"
                            <input
                                type="datetime-local"
                                class="input input-bordered input-sm"
                                prop:value=ends_at
                                on:input=move |event| set_ends_at.set(event_target_value(&event))
                            />
                        </label>
                        <span class="text-xs opacity-50">"Leave empty to keep it up until withdrawn."</span>
                        <button
                            class="btn btn-primary btn-sm ml-auto"
                            disabled=move || busy.get()
                            on:click=move |_| publish(())
                        >
                            "Publish"
                        </button>
                    </div>
                </div>
            </div>

            <Show when=move || error.get().is_some()>
                <p class="text-sm text-error">{move || error.get().unwrap_or_default()}</p>
            </Show>

            <For each=move || items.get() key=|item| item.id.clone() let:item>
                <AnnouncementRow item=item withdrawn=Callback::new(move |()| alive.if_alive(load)) set_error=set_error/>
            </For>

            <Show when=move || items.get().is_empty() && error.get().is_none()>
                <p class="text-sm opacity-60">"No announcements yet."</p>
            </Show>
        </div>
    }
}

/// One announcement, with the button that withdraws it.
#[component]
fn AnnouncementRow(
    item: Announcement,
    withdrawn: Callback<()>,
    set_error: WriteSignal<Option<String>>,
) -> impl IntoView {
    let id = item.id;
    let published = item.published;
    let ends_label = item.ends_at.unwrap_or_default();
    let has_end = !ends_label.is_empty();
    let content = item.content;
    let remove = move |_| {
        let id = id.clone();
        spawn_local(async move {
            match admin::delete_announcement(&id).await {
                Ok(()) => withdrawn.run(()),
                Err(message) => set_error.set(Some(message)),
            }
        });
    };

    view! {
        <article class="card border border-base-300 bg-base-100 shadow-sm">
            <div class="card-body gap-1 p-4">
                <p class="whitespace-pre-line text-sm leading-relaxed">
                    {crate::routes::feed::posts::plain_text(&content)}
                </p>
                <div class="mt-1 flex items-center gap-2 text-xs opacity-60">
                    <Show
                        when=move || published
                        fallback=|| view! { <span class="badge badge-ghost badge-sm">"draft"</span> }
                    >
                        <span class="badge badge-success badge-sm">"published"</span>
                    </Show>
                    <Show when=move || has_end>
                        <span>{ends_label.clone()}</span>
                    </Show>
                    <button class="btn btn-ghost btn-xs ml-auto" on:click=remove>
                        "Withdraw"
                    </button>
                </div>
            </div>
        </article>
    }
}
