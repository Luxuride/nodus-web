//! An account a visitor reached from a shared post: its header, posts, and graph.

use std::sync::Arc;

use leptos::prelude::*;
use nodus_instance::model::Account;

use crate::components::CardRenderer;
use crate::lists::account_row::AccountRow;
use crate::relative_age;

use super::super::Runtime;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Posts,
    Followers,
    Following,
}

impl Tab {
    fn label(self) -> &'static str {
        match self {
            Self::Posts => "Posts",
            Self::Followers => "Followers",
            Self::Following => "Following",
        }
    }
}

#[component]
#[expect(
    clippy::too_many_lines,
    reason = "the header, tabs, posts and two graph lists share one screen"
)]
pub fn PublicAccountScreen(
    runtime: Arc<dyn Runtime>,
    host: String,
    id: String,
    card: CardRenderer,
) -> impl IntoView {
    let runtime = StoredValue::new(runtime);
    let account = RwSignal::new(runtime.get_value().public_account(&host, &id));
    let statuses = RwSignal::new(runtime.get_value().public_account_statuses(&host, &id));
    let followers = RwSignal::new(runtime.get_value().public_account_followers(&host, &id));
    let following = RwSignal::new(runtime.get_value().public_account_following(&host, &id));
    let tab = RwSignal::new(Tab::Posts);
    let error = RwSignal::new(None::<String>);
    let alive = crate::Alive::new();

    let load_host = host.clone();
    let load_id = id.clone();
    let effect_host = host.clone();
    let effect_id = id.clone();
    Effect::new(move |_| {
        if account.get_untracked().is_some() && !statuses.get_untracked().is_empty() {
            return;
        }
        let browse_host = effect_host.clone();
        let browse_id = effect_id.clone();
        let done = Callback::new(move |result: Result<Account, String>| {
            alive.if_alive(|| match result {
                Ok(found) => {
                    runtime.get_value().note_browse(
                        "profile",
                        browse_host.clone(),
                        browse_id.clone(),
                        Some(found.acct.clone()),
                    );
                    account.set(Some(found));
                }
                Err(message) => error.set(Some(message)),
            });
        });
        runtime
            .get_value()
            .load_public_account(load_host.clone(), load_id.clone(), done);
        let done = Callback::new(
            move |result: Result<Vec<nodus_instance::model::Status>, String>| {
                alive.if_alive(|| match result {
                    Ok(found) => statuses.set(found),
                    Err(message) => error.set(Some(message)),
                });
            },
        );
        runtime
            .get_value()
            .load_public_account_statuses(load_host.clone(), load_id.clone(), done);
    });

    let graph_host = host.clone();
    let graph_id = id.clone();
    Effect::new(move |_| match tab.get() {
        Tab::Posts => {}
        Tab::Followers => {
            if followers.get_untracked().is_some() {
                return;
            }
            let done = Callback::new(move |result: Result<Vec<Account>, String>| {
                alive.if_alive(|| match result {
                    Ok(found) => followers.set(Some(found)),
                    Err(message) => error.set(Some(message)),
                });
            });
            runtime
                .get_value()
                .load_public_account_followers(graph_host.clone(), graph_id.clone(), done);
        }
        Tab::Following => {
            if following.get_untracked().is_some() {
                return;
            }
            let done = Callback::new(move |result: Result<Vec<Account>, String>| {
                alive.if_alive(|| match result {
                    Ok(found) => following.set(Some(found)),
                    Err(message) => error.set(Some(message)),
                });
            });
            runtime
                .get_value()
                .load_public_account_following(graph_host.clone(), graph_id.clone(), done);
        }
    });

    let age_of = Callback::new(move |created: String| {
        relative_age(&created, runtime.get_value().now_millis())
    });
    let statuses_view = Signal::derive(move || statuses.get());
    let card = StoredValue::new(card);
    let age_of = StoredValue::new(age_of);
    let link_host = host.clone();
    let active = Signal::derive(move || tab.get().label().to_owned());
    let tabs: ChildrenFn = Arc::new(move || {
        view! {
            <crate::chrome::ButtonTabList
                labels=vec!["Posts".to_owned(), "Followers".to_owned(), "Following".to_owned()]
                active=active
                on_select=Callback::new(move |label: String| {
                    tab.set(match label.as_str() {
                        "Followers" => Tab::Followers,
                        "Following" => Tab::Following,
                        _ => Tab::Posts,
                    });
                })
            />
        }
        .into_any()
    });
    let list_host = link_host;
    let list_id = id.clone();
    let rows: ChildrenFn = Arc::new(move || {
        if tab.get() == Tab::Posts {
            let host = list_host.clone();
            let id = list_id.clone();
            let more = move |_| {
                let Some(last) = statuses.get_untracked().last().map(|s| s.id.clone()) else {
                    return;
                };
                let host = host.clone();
                let id = id.clone();
                let done = Callback::new(move |result: Result<Vec<_>, String>| {
                    alive.if_alive(|| {
                        if let Err(message) = result {
                            error.set(Some(message));
                        }
                    });
                });
                runtime
                    .get_value()
                    .load_public_account_statuses_before(host, id, last, done);
            };
            return view! {
                <button class="btn btn-ghost btn-sm w-full" on:click=more>"Load older posts"</button>
            }
            .into_any();
        }
        let shown = match tab.get() {
            Tab::Posts => Vec::new(),
            Tab::Followers => followers.get().unwrap_or_default(),
            Tab::Following => following.get().unwrap_or_default(),
        };
        let host = list_host.clone();
        view! {
            <ul class="flex flex-col gap-2">
                {shown.into_iter().map(|found| {
                    let target = format!("/accounts/{host}/{}", found.id);
                    view! {
                        <AccountRow
                            note=found.note.clone()
                            account=found
                            on_open=Callback::new(move |_id: String| {
                                runtime.get_value().navigate(target.clone());
                            })
                        />
                    }
                }).collect_view()}
            </ul>
        }
        .into_any()
    });

    let tabs = StoredValue::new(tabs);
    let rows = StoredValue::new(rows);

    view! {
        <Show
            when=move || account.get().is_some()
            fallback=move || {
                view! {
                    <crate::Screen>
                        <Show when=move || error.get().is_some()>
                            <crate::Note class="text-error text-xs">
                                {move || error.get().unwrap_or_default()}
                            </crate::Note>
                        </Show>
                        <crate::chrome::Skeleton count=4/>
                    </crate::Screen>
                }
                .into_any()
            }
        >
            {move || account.get().map(|profile| view! {
                <crate::profile::ProfilePage
                    account=profile
                    relationship=None
                    statuses=statuses_view
                    show_statuses=tab.get() == Tab::Posts
                    show_verbs=false
                    tabs=tabs.get_value()
                    after=rows.get_value()
                    card=card.get_value()
                    age_of=age_of.get_value()
                />
            })}
        </Show>
    }
}
