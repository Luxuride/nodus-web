//! Featured tags: the profile's own table of contents.

mod add;

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::FeaturedTag;

use crate::state::alive::Alive;
use crate::state::bridge::actions;

use self::add::AddTag;

/// Lists, adds and removes the tags on the account's profile.
#[component]
pub fn FeaturedTagsCard() -> impl IntoView {
    let (tags, set_tags) = signal(Vec::<FeaturedTag>::new());
    let (busy, set_busy) = signal(false);
    let (error, set_error) = signal(None::<String>);
    let alive = Alive::new();

    let reload = move || {
        let alive = alive;
        let Some(id) = actions::viewer_id() else {
            return;
        };
        spawn_local(async move {
            match actions::featured_tags(&id).await {
                Ok(list) => alive.if_alive(|| set_tags.set(list)),
                Err(message) => alive.if_alive(|| set_error.set(Some(message))),
            }
        });
    };

    Effect::new(move |_| reload());

    let remove = move |id: String| {
        set_busy.set(true);
        spawn_local(async move {
            match actions::unfeature_tag(&id).await {
                Ok(()) => {
                    alive.if_alive(|| set_busy.set(false));
                    reload();
                }
                Err(message) => alive.if_alive(|| {
                    set_error.set(Some(message));
                    set_busy.set(false);
                }),
            }
        });
    };

    view! {
        <section class="card bg-base-100">
            <div class="card-body gap-3">
                <h2 class="card-title text-base">"Featured tags"</h2>
                <p class="text-sm opacity-70">
                    "Tags on your profile, so a visitor sees what you post about."
                </p>

                <Show when=move || tags.get().is_empty() fallback=|| ()>
                    <p class="text-sm opacity-60">"No featured tags yet."</p>
                </Show>

                <ul class="flex flex-wrap gap-2">
                    <For each=move || tags.get() key=|tag| tag.id.clone() let:tag>
                        {
                            let id = tag.id.clone();
                            view! {
                                <li class="badge badge-lg badge-outline gap-1">
                                    <span>{tag.hashtag()}</span>
                                    <button
                                        class="btn btn-ghost btn-xs"
                                        disabled=move || busy.get()
                                        on:click=move |_| remove(id.clone())
                                    >
                                        "×"
                                    </button>
                                </li>
                            }
                        }
                    </For>
                </ul>

                <AddTag busy=busy set_busy=set_busy set_error=set_error on_added=Callback::new(move |()| reload())/>

                {move || error.get().map(|message| view! {
                    <p class="text-sm text-error">{message}</p>
                })}
            </div>
        </section>
    }
}
