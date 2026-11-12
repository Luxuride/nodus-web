//! The browser side of the shared route renderer.
//!
//! Reads come from the SDK store through the bridge; writes happen only when the route
//! asks for a page that is missing or stale. The renderer never talks to the SDK.

use std::sync::Arc;

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::{Account, Context, Relationship, Status};
use nodus_pages::pages::settings::nav::SettingsSection;
use nodus_route::route::{
    Action, ActionResult, FeedKind, FeedPeopleView, FeedPostsView, Runtime, SessionView,
    TimelineKind, TimelineReply,
};
use nodus_sdk::page::{PageEntry, PageKind};
use nodus_sdk::{Command, Timeline};
use nodus_shared::{AccountItem, PostItem};
use nodus_web::shared::router::Route;

use crate::state::bridge::{
    actions, dispatch, feed_ui, load_posts_once, pages, posts_ui, reload_feed, reload_posts,
    revision, timeline, with_sdk,
};

type PublicAccount = (Account, Vec<Status>);

thread_local! {
    static LAST_PATH: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
    static PUBLIC_ACCOUNT: std::cell::RefCell<
        std::collections::HashMap<(String, String), PublicAccount>,
    > = std::cell::RefCell::new(std::collections::HashMap::new());
    static PUBLIC_EDGES: std::cell::RefCell<
        std::collections::HashMap<(String, String, bool), Vec<Account>>,
    > = std::cell::RefCell::new(std::collections::HashMap::new());
}

/// Remembers the screen under a More page, so its section stays marked while the page is open.
pub fn remember_path(path: &str) {
    if path != Route::More.path() {
        LAST_PATH.with(|last| *last.borrow_mut() = path.to_owned());
    }
}

/// The live runtime both a signed-in and a signed-out tree use.
pub fn runtime() -> Arc<dyn Runtime> {
    Arc::new(ClientRuntime)
}

struct ClientRuntime;

impl Runtime for ClientRuntime {
    fn revision(&self) -> Signal<u64> {
        Signal::from(revision())
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
            view! {
                <crate::routes::feed::PostCard
                    item
                    on_open_media=Some(Callback::new(|(pictures, index)| {
                        crate::media::expand(pictures, index);
                    }))
                />
            }
            .into_any()
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

    fn public_account(&self, host: &str, id: &str) -> Option<Account> {
        let key = (host.to_owned(), id.to_owned());
        PUBLIC_ACCOUNT.with(|cache| cache.borrow().get(&key).map(|(account, _)| account.clone()))
    }

    fn load_public_account(
        &self,
        host: String,
        id: String,
        done: Callback<Result<Account, String>>,
    ) {
        let key = (host.clone(), id.clone());
        spawn_local(async move {
            let result = crate::public::account(&host, &id)
                .await
                .map_err(|failure| failure.message());
            if let Ok(account) = &result {
                PUBLIC_ACCOUNT.with(|cache| {
                    cache
                        .borrow_mut()
                        .entry(key)
                        .or_insert_with(|| (account.clone(), Vec::new()))
                        .0 = account.clone();
                });
                revision().update(|n| *n += 1);
            }
            done.run(result);
        });
    }

    fn public_account_statuses(&self, host: &str, id: &str) -> Vec<Status> {
        let key = (host.to_owned(), id.to_owned());
        PUBLIC_ACCOUNT.with(|cache| {
            cache
                .borrow()
                .get(&key)
                .map(|(_, statuses)| statuses.clone())
                .unwrap_or_default()
        })
    }

    fn load_public_account_statuses(
        &self,
        host: String,
        id: String,
        done: Callback<Result<Vec<Status>, String>>,
    ) {
        let key = (host.clone(), id.clone());
        spawn_local(async move {
            let result = crate::public::account_statuses(&host, &id)
                .await
                .map_err(|failure| failure.message());
            if let Ok(statuses) = &result {
                PUBLIC_ACCOUNT.with(|cache| {
                    if let Some((_, held)) = cache.borrow_mut().get_mut(&key) {
                        *held = statuses.clone();
                    }
                });
                revision().update(|n| *n += 1);
            }
            done.run(result);
        });
    }

    fn load_public_account_statuses_before(
        &self,
        host: String,
        id: String,
        max_id: String,
        done: Callback<Result<Vec<Status>, String>>,
    ) {
        let key = (host.clone(), id.clone());
        spawn_local(async move {
            let result = crate::public::account_statuses_before(&host, &id, &max_id)
                .await
                .map_err(|failure| failure.message());
            if let Ok(older) = &result {
                PUBLIC_ACCOUNT.with(|cache| {
                    if let Some((_, held)) = cache.borrow_mut().get_mut(&key) {
                        held.extend(older.iter().cloned());
                    }
                });
                revision().update(|n| *n += 1);
            }
            done.run(result);
        });
    }

    fn public_account_followers(&self, host: &str, id: &str) -> Option<Vec<Account>> {
        PUBLIC_EDGES.with(|cache| {
            cache
                .borrow()
                .get(&(host.to_owned(), id.to_owned(), true))
                .cloned()
        })
    }

    fn load_public_account_followers(
        &self,
        host: String,
        id: String,
        done: Callback<Result<Vec<Account>, String>>,
    ) {
        let key = (host.clone(), id.clone(), true);
        spawn_local(async move {
            let result = crate::public::account_followers(&host, &id)
                .await
                .map_err(|failure| failure.message());
            if let Ok(accounts) = &result {
                PUBLIC_EDGES.with(|cache| {
                    cache.borrow_mut().insert(key, accounts.clone());
                });
                revision().update(|n| *n += 1);
            }
            done.run(result);
        });
    }

    fn public_account_following(&self, host: &str, id: &str) -> Option<Vec<Account>> {
        PUBLIC_EDGES.with(|cache| {
            cache
                .borrow()
                .get(&(host.to_owned(), id.to_owned(), false))
                .cloned()
        })
    }

    fn load_public_account_following(
        &self,
        host: String,
        id: String,
        done: Callback<Result<Vec<Account>, String>>,
    ) {
        let key = (host.clone(), id.clone(), false);
        spawn_local(async move {
            let result = crate::public::account_following(&host, &id)
                .await
                .map_err(|failure| failure.message());
            if let Ok(accounts) = &result {
                PUBLIC_EDGES.with(|cache| {
                    cache.borrow_mut().insert(key, accounts.clone());
                });
                revision().update(|n| *n += 1);
            }
            done.run(result);
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

    fn reply_composer(&self, parent: String, replies: RwSignal<Vec<Status>>) -> Option<ChildrenFn> {
        Some(Arc::new(move || {
            view! {
                <crate::routes::status::reply::ReplyComposer
                    parent=parent.clone()
                    replies=replies
                />
            }
            .into_any()
        }))
    }

    fn note_browse(
        &self,
        kind: &'static str,
        host: String,
        id: String,
        acct: Option<String>,
        url: Option<String>,
    ) {
        crate::state::bridge::backfill::request(kind, host, id, acct);
        if let Some(url) = url {
            crate::state::bridge::backfill::hydrate(url);
        }
    }

    fn public_host(&self) -> Option<String> {
        web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.query_selector("meta[name=nodus-instance-host]").ok().flatten())
            .and_then(|meta| meta.get_attribute("content"))
            .filter(|host| !host.trim().is_empty())
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

    fn body(&self, route: &Route) -> Option<ChildrenFn> {
        match route {
            Route::Compose => Some(Arc::new(|| {
                view! { <crate::routes::compose::Compose/> }.into_any()
            })),
            Route::ComposeEdit(id) => {
                let id = id.clone();
                Some(Arc::new(move || {
                    view! { <crate::routes::compose::ComposeEdit id=id.clone()/> }.into_any()
                }))
            }
            #[cfg(feature = "admin")]
            Route::Admin(section) => {
                let section = section.clone();
                Some(Arc::new(move || {
                    view! { <crate::routes::admin::Admin section=section.clone()/> }.into_any()
                }))
            }
            #[cfg(feature = "admin")]
            Route::Moderation(section) => {
                let section = section.clone();
                Some(Arc::new(move || {
                    view! { <crate::routes::moderation::Moderation section=section.clone()/> }
                        .into_any()
                }))
            }
            _ => None,
        }
    }

    fn settings_body(&self, section: SettingsSection) -> Option<ChildrenFn> {
        Some(Arc::new(move || {
            view! { <crate::routes::settings::SettingsBody section/> }.into_any()
        }))
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

    fn last_path(&self) -> Option<String> {
        LAST_PATH.with(|last| {
            let path = last.borrow();
            (!path.is_empty()).then(|| path.clone())
        })
    }

    fn navigate(&self, path: String) {
        crate::router::navigate(crate::router::Route::from_path(&path));
    }

    fn infinite_more(&self, active: Signal<bool>, on_load: Callback<()>) -> Option<ChildrenFn> {
        Some(std::sync::Arc::new(move || {
            view! { <crate::ui::infinite::Infinite active=active on_load=on_load/> }.into_any()
        }))
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
