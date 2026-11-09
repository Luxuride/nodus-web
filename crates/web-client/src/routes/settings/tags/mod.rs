//! Featured tags: the profile's own table of contents.

mod add;

use std::sync::Arc;

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_sdk::page::{PageItems, PageKind};
use nodus_pages::pages::settings::lists::{FeaturedTagsCard as SharedFeaturedTagsCard, TagItem};

use crate::state::alive::Alive;
use crate::state::bridge::{actions, pages, revision};

use self::add::AddTag;

/// Lists, adds and removes the tags on the account's profile.
#[component]
pub fn FeaturedTagsCard() -> impl IntoView {
    let revision = revision();
    // The list is the SDK's, not this component's: opening Settings again paints what the
    // store already holds, and only a page past its TTL asks the instance.
    let tags = Signal::derive(move || {
        revision.get();
        pages::items(&PageKind::FeaturedTags, |items| match items {
            PageItems::FeaturedTags(list) => Some(list),
            _ => None,
        })
        .map(|(list, _)| list)
        .unwrap_or_default()
        .into_iter()
        .map(|tag| {
            let label = tag.hashtag();
            TagItem { id: tag.id, label }
        })
        .collect()
    });
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

    let remove = Callback::new(move |id: String| {
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
    });

    let add_form: ChildrenFn = Arc::new(move || {
        view! {
            <AddTag
                busy=busy
                set_busy=set_busy
                set_error=set_error
                on_added=Callback::new(move |()| reload())
            />
        }
        .into_any()
    });

    view! {
        <SharedFeaturedTagsCard
            tags=tags
            busy=Signal::derive(move || busy.get())
            error=Signal::derive(move || error.get())
            on_delete=Some(remove)
            add_form=add_form
        />
    }
}
