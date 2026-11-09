//! One account: who they are, whether we follow them, and what they posted.

use std::sync::Arc;

use leptos::prelude::*;
use nodus_instance::model::{Account, Relationship};

use crate::chrome::{ButtonTabList, LoadMore};
use crate::components::CardRenderer;
use crate::pages::people::Kind;

use super::super::{Action, ActionResult, Runtime, TimelineKind};
use super::people::PeopleList;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Posts,
    Followers,
    Following,
}

impl Tab {
    const ALL: [Self; 3] = [Self::Posts, Self::Followers, Self::Following];

    fn label(self) -> &'static str {
        match self {
            Self::Posts => "Posts",
            Self::Followers => "Followers",
            Self::Following => "Following",
        }
    }

    fn of(label: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|which| which.label() == label)
            .unwrap_or(Self::Posts)
    }
}

/// A profile page.
#[component]
#[expect(
    clippy::too_many_lines,
    reason = "a route screen owns its layout and its reads"
)]
pub fn AccountScreen(runtime: Arc<dyn Runtime>, id: String, card: CardRenderer) -> impl IntoView {
    let runtime = StoredValue::new(runtime);
    let card = StoredValue::new(card);
    let account = RwSignal::new(runtime.get_value().account(&id));
    let relationship = RwSignal::new(runtime.get_value().relationship(&id));
    let tab = RwSignal::new(Tab::Posts);
    let (loading, set_loading) = signal(true);
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let alive = crate::Alive::new();

    let kind = StoredValue::new(TimelineKind::Account(id.clone()));
    let snapshot = Signal::derive(move || {
        runtime.get_value().revision().get();
        runtime.get_value().timeline(&kind.get_value())
    });
    let statuses = Signal::derive(move || snapshot.get().items);
    let cursor = Signal::derive(move || snapshot.get().cursor);

    let load_posts = Callback::new(move |after: Option<String>| {
        set_loading.set(true);
        let done = Callback::new(move |result: Result<(), String>| {
            alive.if_alive(|| {
                if let Err(message) = result {
                    error.set(Some(message));
                }
                set_loading.set(false);
            });
        });
        runtime
            .get_value()
            .load_timeline(kind.get_value(), after, done);
    });

    let load_id = id.clone();
    Effect::new(move |_| {
        if account.get_untracked().is_none() {
            let account_done = Callback::new(move |result: Result<Account, String>| {
                alive.if_alive(|| match result {
                    Ok(found) => account.set(Some(found)),
                    Err(message) => error.set(Some(message)),
                });
            });
            let relationship_done =
                Callback::new(move |result: Result<Option<Relationship>, String>| {
                    alive.if_alive(|| match result {
                        Ok(found) => relationship.set(found),
                        Err(message) => error.set(Some(message)),
                    });
                });
            runtime
                .get_value()
                .load_profile(load_id.clone(), account_done, relationship_done);
        }
        let held = snapshot.get_untracked();
        if !held.held && !held.unavailable {
            load_posts.run(None);
        }
    });

    let tabs: ChildrenFn = Arc::new(move || {
        view! {
            <ButtonTabList
                labels=Tab::ALL.iter().map(|tab| tab.label().to_owned()).collect()
                active=Signal::derive(move || tab.get().label().to_owned())
                on_select=Callback::new(move |label: String| tab.set(Tab::of(&label)))
            />
        }
        .into_any()
    });

    let page_id = StoredValue::new(id);
    let after: ChildrenFn = Arc::new(move || {
        view! {
            <Show when=move || tab.get() == Tab::Posts>
                <Show when=move || cursor.get().is_some()>
                    <LoadMore
                        loading=Signal::derive(move || loading.get())
                        on_more=Callback::new(move |()| load_posts.run(cursor.get_untracked()))
                    />
                </Show>
            </Show>
            <Show when=move || tab.get() == Tab::Followers>
                <PeopleList
                    runtime=runtime.get_value()
                    id=page_id.get_value()
                    kind=Kind::Followers
                />
            </Show>
            <Show when=move || tab.get() == Tab::Following>
                <PeopleList
                    runtime=runtime.get_value()
                    id=page_id.get_value()
                    kind=Kind::Following
                />
            </Show>
            <Show when=move || error.get().is_some()>
                <crate::Note class="text-error text-xs">
                    {move || error.get().unwrap_or_default()}
                </crate::Note>
            </Show>
        }
        .into_any()
    });

    let age_of = StoredValue::new(Callback::new(move |created_at: String| {
        crate::relative_age(&created_at, runtime.get_value().now_millis())
    }));

    let on_avatar = Callback::new(move |url: String| {
        runtime.get_value().action(
            Action::OpenMedia {
                urls: vec![url],
                index: 0,
            },
            Callback::new(|_| ()),
        );
    });

    view! {
        {move || account.get().map(|profile| {
            let id = profile.id.clone();
            view! {
                <crate::profile::ProfilePage
                    account=profile
                    relationship=relationship.get()
                    statuses=statuses
                    show_statuses=tab.get() == Tab::Posts
                    tabs=tabs.clone()
                    after=after.clone()
                    card=card.get_value()
                    age_of=age_of.get_value()
                    on_follow=Callback::new({
                        let id = id.clone();
                        move |(_, on): (String, bool)| {
                            toggle(
                                &runtime.get_value(),
                                Action::Follow { id: id.clone(), on },
                                relationship,
                                busy,
                                error,
                                alive,
                            );
                        }
                    })
                    on_mute=Callback::new({
                        let id = id.clone();
                        move |(_, on): (String, bool)| {
                            toggle(
                                &runtime.get_value(),
                                Action::Mute { id: id.clone(), on },
                                relationship,
                                busy,
                                error,
                                alive,
                            );
                        }
                    })
                    on_block=Callback::new({
                        let id = id.clone();
                        move |(_, on): (String, bool)| {
                            toggle(
                                &runtime.get_value(),
                                Action::Block { id: id.clone(), on },
                                relationship,
                                busy,
                                error,
                                alive,
                            );
                        }
                    })
                    on_avatar=on_avatar
                />
            }
        })}
    }
}

fn toggle(
    runtime: &Arc<dyn Runtime>,
    action: Action,
    relationship: RwSignal<Option<Relationship>>,
    busy: RwSignal<bool>,
    error: RwSignal<Option<String>>,
    alive: crate::Alive,
) {
    if busy.get_untracked() || relationship.get_untracked().is_none() {
        return;
    }
    busy.set(true);
    error.set(None);
    let done = Callback::new(move |result: Result<ActionResult, String>| {
        alive.if_alive(|| {
            match result {
                Ok(ActionResult::Relationship(found)) => relationship.set(Some(found)),
                Ok(_) => {}
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    });
    runtime.action(action, done);
}
