//! The browser side of the shared route renderer.
//!
//! Reads come from the SDK store through the bridge; writes happen only when the route
//! asks for a page that is missing or stale. The renderer never talks to the SDK.

use std::sync::Arc;

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::{Account, Context, Relationship, Status};
use nodus_sdk::page::{PageEntry, PageKind};
use nodus_sdk::{Command, Timeline};
use nodus_shared::{AccountItem, PostItem};
use nodus_view::route::{
    Action, ActionResult, FeedKind, FeedPeopleView, FeedPostsView, Runtime, SessionView,
    TimelineKind, TimelineReply,
};

use crate::state::bridge::{
    actions, dispatch, feed_ui, load_posts_once, pages, posts_ui, reload_feed, reload_posts,
    revision, timeline, with_sdk,
};

/// The live runtime both a signed-in and a signed-out tree use.
pub fn runtime() -> Arc<dyn Runtime> {
    Arc::new(ClientRuntime)
}

struct ClientRuntime;

impl Runtime for ClientRuntime {
    fn revision(&self) -> Signal<u64> {
        revision().into()
    }

    fn timeline(&self, kind: &TimelineKind) -> TimelineReply {
        let which = which(kind);
        let (items, cursor) = timeline::read(which.clone(), None);
        TimelineReply {
            items,
            cursor,
            held: timeline::held(which, None),
            unavailable: false,
        }
    }

    fn load_timeline(
        &self,
        kind: TimelineKind,
        cursor: Option<String>,
        done: Callback<Result<(), String>>,
    ) {
        let which = which(&kind);
        spawn_local(async move {
            done.run(timeline::ensure(which, None, cursor).await.map(|_| ()));
        });
    }

    fn page(&self, kind: &PageKind) -> Option<PageEntry> {
        pages::read(kind)
    }

    fn load_page(
        &self,
        kind: PageKind,
        cursor: Option<String>,
        done: Callback<Result<(), String>>,
    ) {
        spawn_local(async move {
            done.run(pages::ensure_after(kind, cursor).await.map(|_| ()));
        });
    }

    fn load_thread(
        &self,
        id: String,
        status: Callback<Result<Status, String>>,
        context: Callback<Result<Context, String>>,
    ) {
        let for_status = id.clone();
        spawn_local(async move {
            let result = actions::status(&for_status).await;
            if let Ok(found) = &result {
                timeline::upsert(vec![found.clone()]);
            }
            status.run(result);
        });
        spawn_local(async move {
            let result = actions::thread(&id).await;
            if let Ok(found) = &result {
                let mut all = found.ancestors.clone();
                all.extend(found.descendants.clone());
                timeline::upsert(all);
            }
            context.run(result);
        });
    }

    fn feed_people(&self) -> FeedPeopleView {
        let revision = revision();
        let ui = feed_ui();
        with_sdk(|sdk| {
            revision.get();
            let state = sdk.state().feed.get();
            FeedPeopleView {
                items: state.visible().cloned().collect(),
                degraded: state
                    .degraded
                    .first()
                    .map(|flag| crate::routes::feed::degraded_line(*flag).to_owned()),
                cursor: state.next_cursor,
                error: ui.error.get(),
                refreshing: ui.refreshing.get(),
            }
        })
    }

    fn feed_posts(&self) -> FeedPostsView {
        let revision = revision();
        let ui = posts_ui();
        with_sdk(|sdk| {
            revision.get();
            let state = sdk.state().posts.get();
            FeedPostsView {
                items: state.items,
                cursor: state.next_cursor,
                error: ui.error.get(),
                refreshing: ui.refreshing.get(),
            }
        })
    }

    fn load_feed(
        &self,
        kind: FeedKind,
        cursor: Option<String>,
        done: Callback<Result<(), String>>,
    ) {
        match kind {
            FeedKind::People => {
                reload_feed(cursor);
                done.run(Ok(()));
            }
            FeedKind::Posts => {
                if cursor.is_none() {
                    load_posts_once();
                } else {
                    reload_posts(cursor);
                }
                done.run(Ok(()));
            }
        }
    }

    fn feed_people_row(&self) -> Option<Callback<AccountItem, AnyView>> {
        Some(Callback::new(|item| {
            view! { <crate::routes::feed::FeedItem item/> }.into_any()
        }))
    }

    fn feed_posts_row(&self) -> Option<Callback<PostItem, AnyView>> {
        Some(Callback::new(|item| {
            view! { <crate::routes::feed::PostCard item/> }.into_any()
        }))
    }

    fn feed_settings(&self) -> Option<Callback<()>> {
        Some(Callback::new(|()| {
            crate::router::navigate(crate::router::Route::Settings(
                crate::router::SECTION_DEFAULT.to_owned(),
            ));
        }))
    }

    fn load_public_thread(
        &self,
        host: String,
        id: String,
        status: Callback<Result<Status, String>>,
        context: Callback<Result<Context, String>>,
    ) {
        let for_status = (host.clone(), id.clone());
        spawn_local(async move {
            let result = crate::public::status(&for_status.0, &for_status.1)
                .await
                .map_err(|failure| failure.message());
            status.run(result);
        });
        spawn_local(async move {
            let result = crate::public::context(&host, &id)
                .await
                .map_err(|failure| failure.message());
            context.run(result);
        });
    }

    fn load_profile(
        &self,
        id: String,
        account: Callback<Result<Account, String>>,
        relationship: Callback<Result<Option<Relationship>, String>>,
    ) {
        let for_account = id.clone();
        spawn_local(async move {
            account.run(actions::account(&for_account).await);
        });
        spawn_local(async move {
            relationship.run(actions::relationship(&id).await);
        });
    }

    fn session(&self) -> Option<SessionView> {
        with_sdk(|sdk| {
            sdk.state().session.get().map(|session| SessionView {
                host: session.host,
                account_id: session.account_id,
                acct: session.acct,
            })
        })
    }

    fn follow_requests(&self) -> Option<Vec<Account>> {
        None
    }

    fn load_follow_requests(&self, done: Callback<Result<Vec<Account>, String>>) {
        spawn_local(async move {
            done.run(actions::follow_requests().await);
        });
    }

    fn lists(&self) -> Option<Vec<nodus_instance::model::List>> {
        crate::state::bridge::seed::lists()
    }

    fn load_lists(&self, done: Callback<Result<Vec<nodus_instance::model::List>, String>>) {
        spawn_local(async move {
            done.run(actions::lists().await);
        });
    }

    fn search_accounts(&self, query: String, done: Callback<Result<Vec<Account>, String>>) {
        spawn_local(async move {
            done.run(
                crate::state::bridge::actions::search(&query, true)
                    .await
                    .map(|found| found.accounts),
            );
        });
    }

    fn load_list_accounts(&self, id: String, done: Callback<Result<Vec<Account>, String>>) {
        spawn_local(async move {
            done.run(actions::list_accounts(&id).await);
        });
    }

    fn action(&self, action: Action, done: Callback<Result<ActionResult, String>>) {
        spawn_local(async move {
            let result = match action {
                Action::Favourite { id, on } => {
                    actions::favourite(&id, on).await.map(ActionResult::Status)
                }
                Action::Reblog { id, on } => {
                    actions::reblog(&id, on).await.map(ActionResult::Status)
                }
                Action::Bookmark { id, on } => {
                    actions::bookmark(&id, on).await.map(ActionResult::Status)
                }
                Action::DeleteStatus(id) => actions::delete_status(&id)
                    .await
                    .map(|()| ActionResult::None),
                Action::Vote { poll, choices } => actions::vote(&poll, &choices)
                    .await
                    .map(ActionResult::Status),
                Action::Reply { parent, text } => {
                    actions::publish(&actions::NewStatus::reply(text, &parent))
                        .await
                        .map(ActionResult::Status)
                }
                Action::Follow { id, on } => actions::follow(&id, on)
                    .await
                    .map(ActionResult::Relationship),
                Action::Mute { id, on } => {
                    actions::mute(&id, on).await.map(ActionResult::Relationship)
                }
                Action::Block { id, on } => actions::block(&id, on)
                    .await
                    .map(ActionResult::Relationship),
                Action::AnswerFollowRequest { id, accept } => {
                    if accept {
                        actions::authorize_follow_request(&id)
                            .await
                            .map(ActionResult::Relationship)
                    } else {
                        actions::reject_follow_request(&id)
                            .await
                            .map(|()| ActionResult::None)
                    }
                }
                Action::ReadConversation(id) => actions::mark_conversation_read(&id)
                    .await
                    .map(|()| ActionResult::None),
                Action::MuteConversation { id, on } => actions::mute_conversation(&id, on)
                    .await
                    .map(ActionResult::Conversation),
                Action::CreateList(title) => {
                    actions::create_list(&title).await.map(ActionResult::List)
                }
                Action::RenameList { id, title } => actions::rename_list(&id, &title)
                    .await
                    .map(ActionResult::List),
                Action::DeleteList(id) => {
                    actions::delete_list(&id).await.map(|()| ActionResult::None)
                }
                Action::SetListMember {
                    list,
                    account,
                    member,
                } => actions::set_list_member(&list, &account, member)
                    .await
                    .map(|()| ActionResult::None),
                Action::DeleteFilter(id) => actions::delete_filter(&id)
                    .await
                    .map(|()| ActionResult::None),
                Action::FeatureTag(name) => actions::feature_tag(&name)
                    .await
                    .map(|()| ActionResult::None),
                Action::UnfeatureTag(id) => actions::unfeature_tag(&id)
                    .await
                    .map(|()| ActionResult::None),
                Action::CancelScheduled(id) => actions::cancel_scheduled(&id)
                    .await
                    .map(|()| ActionResult::None),
                Action::OpenMedia { urls, index } => {
                    if let Some(url) = urls.get(index) {
                        crate::media::expand_one(url.clone(), String::new());
                    }
                    Ok(ActionResult::None)
                }
                Action::UpdateProfile { display_name, note } => {
                    actions::update_profile(display_name, note)
                        .await
                        .map(ActionResult::Account)
                }
            };
            if let Ok(ActionResult::Status(status)) = &result {
                dispatch(Command::StatusesUpserted {
                    statuses: vec![status.clone()],
                });
            }
            done.run(result);
        });
    }

    fn now_millis(&self) -> f64 {
        js_sys::Date::now()
    }

    fn query(&self, key: &str) -> Option<String> {
        crate::router::query_param(key)
    }

    fn drop_account(&self, kind: &nodus_sdk::page::PageKind, account: &str) {
        dispatch(Command::PageAccountDropped {
            kind: kind.clone(),
            account: account.to_owned(),
        });
    }

    fn navigate(&self, path: String) {
        crate::router::navigate(crate::router::Route::from_path(&path));
    }
}

fn which(kind: &TimelineKind) -> Timeline {
    match kind {
        TimelineKind::Home => Timeline::Home,
        TimelineKind::Local => Timeline::Local,
        TimelineKind::Federated => Timeline::Federated,
        TimelineKind::Account(id) => Timeline::Account(id.clone()),
        TimelineKind::List(id) => Timeline::List(id.clone()),
    }
}
