//! Featured tags: the profile's own table of contents.

mod add;

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_sdk::page::{PageItems, PageKind};

use crate::state::alive::Alive;
use crate::state::bridge::{actions, pages, revision};

use self::add::AddTag;

/// Lists, adds and removes the tags on the account's profile.
#[component]
pub fn FeaturedTagsCard() -> impl IntoView {
    let revision = revision();
    // The list is the SDK's, not this component's: opening Settings again paints what the
    // store already holds, and only a page past its TTL asks the instance.
    let tags = move || {
        revision.get();
        pages::items(&PageKind::FeaturedTags, |items| match items {
            PageItems::FeaturedTags(list) => Some(list),
            _ => None,
        })
        .map(|(list, _)| list)
        .unwrap_or_default()
    };
    let (busy, set_busy) = signal(false);
    let (error, set_error) = signal(None::<String>);
    let alive = Alive::new();

    // Two verbs: the mount may trust the store, a change may not.
    let read = move || {
        let alive = alive;
        spawn_local(async move {
            if let Err(message) = pages::ensure(PageKind::FeaturedTags).await {
                alive.if_alive(|| set_error.set(Some(message)));
            }
        });
    };
    let reload = move || {
        spawn_local(async move {
            if let Err(message) = pages::refresh(PageKind::FeaturedTags).await {
                alive.if_alive(|| set_error.set(Some(message)));
            }
        });
    };

    Effect::new(move |_| read());

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

                <Show when=move || tags().is_empty() fallback=|| ()>
                    <p class="text-sm opacity-60">"No featured tags yet."</p>
                </Show>

                <ul class="flex flex-wrap gap-2">
                    <For each=tags key=|tag| tag.id.clone() let:tag>
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
