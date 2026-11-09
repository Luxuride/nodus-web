//! What a screen needs from the platform, and the no-op answers the server gives.
//!
//! The renderer asks for one synchronous snapshot per read. The browser answers from the
//! live SDK store; the server answers from the page it prefetched. An operation the server
//! cannot perform is reported as unavailable, so the screen draws its own failure state
//! instead of a second, server-only page.

use std::sync::Arc;

use leptos::prelude::*;
use nodus_instance::model::{Account, Context, Conversation, Filter, List, Relationship, Status};
use nodus_state::page::{PageEntry, PageKind};
use nodus_web::shared::router::Route;

use crate::Alive;
use crate::components::CardRenderer;

pub mod screens;
use screens::account::AccountScreen;
use screens::directory::DirectoryScreen;
use screens::lists::ListsScreen;
use screens::messages::MessagesScreen;
use screens::notifications::NotificationsScreen;
use screens::people::PeopleList;
use screens::saved::SavedScreen;
use screens::scheduled::ScheduledScreen;
use screens::search::SearchScreen;
use screens::thread::ThreadScreen;

/// Which timeline a route asks for.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TimelineKind {
    Home,
    Local,
    Federated,
    Account(String),
    List(String),
}

/// One page of a timeline, as the platform can answer it right now.
#[derive(Clone, Debug, Default)]
pub struct TimelineReply {
    pub items: Vec<Status>,
    pub cursor: Option<String>,
    pub held: bool,
    pub unavailable: bool,
}

impl TimelineReply {
    /// The answer when the stateless platform has no page for this timeline.
    #[must_use]
    pub fn unavailable() -> Self {
        Self {
            unavailable: true,
            ..Self::default()
        }
    }
}

/// The signed-in account, as much as a view needs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionView {
    pub host: String,
    pub account_id: String,
    pub acct: String,
}

/// A change the reader asked for.
#[derive(Clone, Debug)]
pub enum Action {
    Favourite {
        id: String,
        on: bool,
    },
    Reblog {
        id: String,
        on: bool,
    },
    Bookmark {
        id: String,
        on: bool,
    },
    DeleteStatus(String),
    Vote {
        poll: String,
        choices: Vec<usize>,
    },
    Reply {
        parent: String,
        text: String,
    },
    Follow {
        id: String,
        on: bool,
    },
    Mute {
        id: String,
        on: bool,
    },
    Block {
        id: String,
        on: bool,
    },
    AnswerFollowRequest {
        id: String,
        accept: bool,
    },
    ReadConversation(String),
    MuteConversation {
        id: String,
        on: bool,
    },
    CreateList(String),
    RenameList {
        id: String,
        title: String,
    },
    DeleteList(String),
    SetListMember {
        list: String,
        account: String,
        member: bool,
    },
    DeleteFilter(String),
    FeatureTag(String),
    UnfeatureTag(String),
    CancelScheduled(String),
    OpenMedia {
        urls: Vec<String>,
        index: usize,
    },
    UpdateProfile {
        display_name: String,
        note: String,
    },
}

/// What a completed action answered with, when the screen needs the new value.
#[derive(Clone, Debug)]
pub enum ActionResult {
    None,
    Status(Status),
    Conversation(Conversation),
    Relationship(Relationship),
    List(List),
    Filter(Filter),
    Account(Account),
}

/// The platform side of a rendered route.
pub trait Runtime: Send + Sync + 'static {
    /// Bumps whenever a store the screen reads has changed.
    fn revision(&self) -> Signal<u64>;

    /// The current snapshot. Called while rendering; must not start work.
    fn timeline(&self, kind: &TimelineKind) -> TimelineReply;

    /// Starts a read and reports completion. The server answers immediately; the browser
    /// spawns it and calls `done` when the store has been updated.
    fn load_timeline(
        &self,
        kind: TimelineKind,
        cursor: Option<String>,
        done: Callback<Result<(), String>>,
    );

    /// One stored page, if the platform can answer it now.
    fn page(&self, _kind: &PageKind) -> Option<PageEntry> {
        None
    }

    /// Starts a page read. `cursor` is the page the reader is continuing from.
    fn load_page(
        &self,
        _kind: PageKind,
        _cursor: Option<String>,
        done: Callback<Result<(), String>>,
    ) {
        done.run(Err("This page is not available without the app.".to_owned()));
    }

    /// A status already in hand, if the platform keeps one.
    fn status(&self, _id: &str) -> Option<Status> {
        None
    }

    /// A conversation context already in hand.
    fn context(&self, _id: &str) -> Option<Context> {
        None
    }

    /// An account already in hand.
    fn account(&self, _id: &str) -> Option<Account> {
        None
    }

    /// The viewer's relationship to an account.
    fn relationship(&self, _id: &str) -> Option<Relationship> {
        None
    }

    /// Loads one status, then one context.
    fn load_thread(
        &self,
        _id: String,
        _status: Callback<Result<Status, String>>,
        _context: Callback<Result<Context, String>>,
    ) {
    }

    /// Loads a profile and the relationship that decides its verbs.
    fn load_profile(
        &self,
        _id: String,
        _account: Callback<Result<Account, String>>,
        _relationship: Callback<Result<Option<Relationship>, String>>,
    ) {
    }

    /// The signed-in account, if any.
    fn session(&self) -> Option<SessionView> {
        None
    }

    /// Follow requests waiting for an answer.
    fn follow_requests(&self) -> Option<Vec<Account>> {
        None
    }

    /// Loads the follow-request queue.
    fn load_follow_requests(&self, _done: Callback<Result<Vec<Account>, String>>) {}

    /// The lists the viewer owns.
    fn lists(&self) -> Option<Vec<List>> {
        None
    }

    /// Loads the list collection.
    fn load_lists(&self, _done: Callback<Result<Vec<List>, String>>) {}

    /// The accounts on one list, as last read.
    fn list_accounts(&self, _id: &str) -> Option<Vec<Account>> {
        None
    }

    /// Loads the accounts on one list.
    fn load_list_accounts(&self, _id: String, _done: Callback<Result<Vec<Account>, String>>) {}

    /// Performs one reader action. A stateless paint refuses it.
    /// Search accounts for a list's add form.
    fn search_accounts(&self, _query: String, done: Callback<Result<Vec<Account>, String>>) {
        done.run(Err("Search needs the app.".to_owned()));
    }

    fn action(&self, _action: Action, done: Callback<Result<ActionResult, String>>) {
        done.run(Err(
            "This action is not available without the app.".to_owned()
        ));
    }

    /// The clock both paints use for post ages.
    fn now_millis(&self) -> f64;

    /// The query string argument a route was opened with.
    fn query(&self, _key: &str) -> Option<String> {
        None
    }

    /// A destination click. The server has nowhere to navigate and leaves the default.
    fn drop_account(&self, _kind: &PageKind, _account: &str) {}

    fn navigate(&self, _path: String) {}
}

/// The callback a stateless paint supplies for one verb: present, but inert.
#[must_use]
pub fn noop<In>() -> Callback<In>
where
    In: Send + Sync + 'static,
{
    Callback::new(|_| ())
}

/// The route renderer both paints call.
#[component]
pub fn RouteView(
    runtime: Arc<dyn Runtime>,
    route: Route,
    card: CardRenderer,
    #[prop(optional_no_strip)] header: Option<ChildrenFn>,
    #[prop(optional_no_strip)] banner: Option<ChildrenFn>,
) -> impl IntoView {
    match route {
        Route::Home | Route::Local | Route::Federated => {
            let kind = match route {
                Route::Home => TimelineKind::Home,
                Route::Local => TimelineKind::Local,
                _ => TimelineKind::Federated,
            };
            view! {
                <TimelineScreen runtime kind header banner card/>
            }
            .into_any()
        }
        Route::Notifications => view! {
            <NotificationsScreen runtime card/>
        }
        .into_any(),
        Route::Messages => view! {
            <MessagesScreen runtime/>
        }
        .into_any(),
        Route::Saved(shelf) => view! {
            <SavedScreen runtime shelf card/>
        }
        .into_any(),
        Route::Directory => view! {
            <DirectoryScreen runtime/>
        }
        .into_any(),
        Route::Scheduled => view! {
            <ScheduledScreen runtime/>
        }
        .into_any(),
        Route::Search => view! {
            <SearchScreen runtime card/>
        }
        .into_any(),
        Route::Thread { id, .. } => view! {
            <ThreadScreen runtime id card/>
        }
        .into_any(),
        Route::Account(id) => view! {
            <AccountScreen runtime id card/>
        }
        .into_any(),
        Route::Lists => view! {
            <ListsScreen runtime card/>
        }
        .into_any(),
        Route::Mutes => view! {
            <PeopleList runtime id=String::new() kind=crate::pages::people::Kind::Mutes/>
        }
        .into_any(),
        Route::Blocks => view! {
            <PeopleList runtime id=String::new() kind=crate::pages::people::Kind::Blocks/>
        }
        .into_any(),
        other => {
            let path = other.path();
            view! {
                <crate::Screen>
                    <crate::PageHeader title="Not available" description=path.clone()/>
                    <crate::Note>
                        "This screen is not available from a stateless render."
                    </crate::Note>
                </crate::Screen>
            }
            .into_any()
        }
    }
}

#[component]
fn TimelineScreen(
    runtime: Arc<dyn Runtime>,
    kind: TimelineKind,
    card: CardRenderer,
    #[prop(optional_no_strip)] header: Option<ChildrenFn>,
    #[prop(optional_no_strip)] banner: Option<ChildrenFn>,
) -> impl IntoView {
    let revision = runtime.revision();
    let initial = runtime.timeline(&kind);
    let loading = RwSignal::new(!initial.held && !initial.unavailable);
    let error = RwSignal::new(
        initial
            .unavailable
            .then(|| "This timeline is not available without the app.".to_owned()),
    );
    let alive = Alive::new();

    let data = Signal::derive({
        let runtime = Arc::clone(&runtime);
        let kind = kind.clone();
        move || {
            revision.get();
            runtime.timeline(&kind)
        }
    });
    let items = Signal::derive(move || data.get().items);
    let more = Signal::derive(move || data.get().cursor.is_some());

    let start = {
        let runtime = Arc::clone(&runtime);
        let kind = kind.clone();
        move |cursor: Option<String>| {
            if loading.get_untracked() {
                return;
            }
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
            runtime.load_timeline(kind.clone(), cursor, done);
        }
    };

    {
        let start = start.clone();
        Effect::new(move |_| {
            if !data.get_untracked().held && !data.get_untracked().unavailable {
                start(None);
            }
        });
    }

    let on_more = Callback::new({
        let start = start.clone();
        move |()| start(data.get_untracked().cursor)
    });
    let (title, description, empty_note) = copy(&kind);

    view! {
        <crate::pages::timeline::TimelinePage
            title=title
            description=description
            empty_note=empty_note
            header=header
            banner=banner
            error=Signal::derive(move || error.get())
            loading=loading
            items=items
            card=card
            more=more
            on_more=Some(on_more)
        />
    }
}

fn copy(kind: &TimelineKind) -> (&'static str, &'static str, &'static str) {
    match kind {
        TimelineKind::Home => (
            "Home",
            "Posts from the people you follow.",
            "The people you follow have not posted yet.",
        ),
        TimelineKind::Local => (
            "Local",
            "Posts from this instance.",
            "No one on this instance has posted yet.",
        ),
        TimelineKind::Federated => (
            "Federated",
            "Posts from instances this one knows about.",
            "No federated posts have arrived yet.",
        ),
        TimelineKind::Account(_) => (
            "Posts",
            "Posts from this account.",
            "This account has not posted yet.",
        ),
        TimelineKind::List(_) => (
            "List",
            "Posts from the accounts on this list.",
            "Nobody on this list has posted yet.",
        ),
    }
}
