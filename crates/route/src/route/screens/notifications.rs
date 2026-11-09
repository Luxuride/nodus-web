//! Notifications: the list and the follow-request queue above it.

use std::sync::Arc;

use leptos::prelude::*;
use nodus_instance::model::{Account, Status};
use nodus_state::page::{PageItems, PageKind};

use crate::components::CardRenderer;
use crate::pages::notifications::{FollowRequestActions, NotificationsPage, RequestBanner};
use crate::relative_age;

use super::super::{Action, ActionResult, Runtime};

#[component]
#[expect(
    clippy::too_many_lines,
    reason = "a route screen owns its layout and its reads"
)]
pub fn NotificationsScreen(runtime: Arc<dyn Runtime>, card: CardRenderer) -> impl IntoView {
    let revision = runtime.revision();
    let initial = runtime.page(&PageKind::Notifications);
    let loading = RwSignal::new(initial.is_none());
    let error = RwSignal::new(None::<String>);
    let alive = crate::Alive::new();

    let items = {
        let runtime = Arc::clone(&runtime);
        move || {
            revision.get();
            runtime
                .page(&PageKind::Notifications)
                .and_then(|page| match page.items {
                    PageItems::Notifications(items) => Some(items),
                    _ => None,
                })
                .unwrap_or_default()
        }
    };
    let cursor = {
        let runtime = Arc::clone(&runtime);
        move || {
            revision.get();
            runtime
                .page(&PageKind::Notifications)
                .and_then(|page| page.cursor)
        }
    };
    let age_of = Callback::new({
        let runtime = Arc::clone(&runtime);
        move |created: String| relative_age(&created, runtime.now_millis())
    });

    let load: Callback<Option<String>> = Callback::new({
        let runtime = Arc::clone(&runtime);
        move |after: Option<String>| {
            error.set(None);
            loading.set(true);
            let done = Callback::new(move |result: Result<(), String>| {
                alive.if_alive(|| {
                    loading.set(false);
                    if let Err(message) = result {
                        error.set(Some(message));
                    }
                });
            });
            runtime.load_page(PageKind::Notifications, after, done);
        }
    });

    {
        let runtime = Arc::clone(&runtime);
        Effect::new(move |_| {
            if runtime.page(&PageKind::Notifications).is_none() {
                load.run(None);
            }
        });
    }

    let on_more = Callback::new(move |after: String| load.run(Some(after)));
    let on_open = Callback::new({
        let runtime = Arc::clone(&runtime);
        move |id: String| runtime.navigate(format!("/status/{id}"))
    });
    let status_card = Callback::new({
        let card = Arc::clone(&card);
        move |status: Status| card(status)
    });

    let (answering, set_answering) = signal(false);
    let requests = RwSignal::new(runtime.follow_requests());
    let answer = Callback::new({
        let runtime = Arc::clone(&runtime);
        move |(id, accept): (String, bool)| {
            set_answering.set(true);
            let done = Callback::new({
                let runtime = Arc::clone(&runtime);
                move |result: Result<ActionResult, String>| {
                    set_answering.set(false);
                    match result {
                        Ok(_) => {
                            // The queue is the instance's answer, not a local guess.
                            let done =
                                Callback::new(move |result: Result<Vec<Account>, String>| {
                                    if let Ok(found) = result {
                                        requests.set(Some(found));
                                    }
                                });
                            runtime.load_follow_requests(done);
                            load.run(None);
                        }
                        Err(message) => error.set(Some(message)),
                    }
                }
            });
            runtime.action(Action::AnswerFollowRequest { id, accept }, done);
        }
    });

    let request_actions = Callback::new({
        move |account_id: String| {
            view! {
                <FollowRequestActions
                    account_id=account_id
                    busy=Signal::derive(move || answering.get())
                    on_answer=answer
                />
            }
            .into_any()
        }
    });

    let banner: ChildrenFn = Arc::new({
        move || {
            view! {
                <RequestBanner
                    requests=Signal::derive(move || requests.get().unwrap_or_default())
                    busy=Signal::derive(move || answering.get())
                    on_answer=answer
                    on_open=Callback::new({
                        let runtime = Arc::clone(&runtime);
                        move |id: String| runtime.navigate(format!("/accounts/{id}"))
                    })
                />
            }
            .into_any()
        }
    });

    view! {
        <NotificationsPage
            notifications=Signal::derive(items)
            cursor=Signal::derive(cursor)
            loading=loading.into()
            error=error.into()
            age_of=age_of
            on_more=on_more
            on_open=on_open
            status_card=status_card
            request_actions=request_actions
            banner=banner
        />
    }
}
