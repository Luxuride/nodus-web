//! One way to feed a page.
//!
//! Every page takes its data as signals and its affordances as `Option<Callback<..>>`. The
//! client hands both; the server, which can only read, hands the data and no callbacks. That
//! split used to be spelled out as a nine-field struct literal on the server side, once per
//! page — and one of those literals built the page *outside* the reactive owner, which
//! aborted the process the first time a list had a row in it.
//!
//! So the server's shape lives here instead: `MessagesPageProps::server(items, ages)` fills
//! the data and writes `None` for every verb. A page that gains a field gains it in one
//! place, and a page that forgets to is a compile error rather than a missing button.

use leptos::prelude::{AnyView, Callback, Signal};
use nodus_client_core::model::{
    Account, Conversation, List, Notification, ScheduledStatus, SearchResults, Status,
};

use crate::components::card_renderer::CardRenderer;
use crate::pages::directory::DirectoryPageProps;
use crate::pages::lists::ListsPageProps;
use crate::pages::messages::MessagesPageProps;
use crate::pages::notifications::NotificationsPageProps;
use crate::pages::saved::SavedPageProps;
use crate::pages::saved::shelf::Shelf;
use crate::pages::scheduled::ScheduledPageProps;
use crate::pages::search::SearchPageProps;

/// A value the caller already has, handed to a component that expects a signal.
pub fn held<T: Clone + Send + Sync + 'static>(value: T) -> Signal<T> {
    Signal::derive(move || value.clone())
}

/// An empty option.
pub fn none<T: Clone + Send + Sync + 'static>() -> Signal<Option<T>> {
    Signal::derive(|| None)
}

/// "Nothing is loading": the server has already finished reading before it paints.
pub fn settled() -> Signal<bool> {
    Signal::derive(|| false)
}

/// The ages a page prints, measured against the clock reading the caller lends it.
pub fn ages(now_millis: f64) -> Callback<String, String> {
    Callback::new(move |created: String| crate::relative_age(&created, now_millis))
}

impl MessagesPageProps {
    /// The server's shape: conversations and a clock, no verbs.
    #[must_use]
    pub fn server(
        conversations: Vec<Conversation>,
        age_of: Option<Callback<String, String>>,
    ) -> Self {
        Self {
            conversations: held(conversations),
            muted: held(Vec::new()),
            loading: settled(),
            error: none(),
            cursor: none(),
            age_of,
            on_open: None,
            on_read: None,
            on_mute: None,
            on_more: None,
        }
    }
}

impl NotificationsPageProps {
    /// The server's shape: notifications, a clock, and a way to paint the post a boost or a
    /// favourite points at.
    #[must_use]
    pub fn server(
        notifications: Vec<Notification>,
        age_of: Option<Callback<String, String>>,
        status_card: Option<Callback<Status, AnyView>>,
    ) -> Self {
        Self {
            notifications: held(notifications),
            cursor: none(),
            loading: settled(),
            error: none(),
            age_of,
            on_more: None,
            on_open: None,
            status_card,
            request_actions: None,
            banner: None,
        }
    }
}

impl ListsPageProps {
    /// The server's shape: the lists only. Members and timelines are the client's panels.
    #[must_use]
    pub fn server(lists: Vec<List>) -> Self {
        Self {
            lists: held(lists),
            selected: none(),
            title: held(String::new()),
            busy: settled(),
            error: none(),
            on_title: None,
            on_create: None,
            row: None,
            members: None,
            timeline: None,
        }
    }
}

impl ScheduledPageProps {
    /// The server's shape: what is queued, and a clock.
    #[must_use]
    pub fn server(items: Vec<ScheduledStatus>, age_of: Option<Callback<String, String>>) -> Self {
        Self {
            items: held(items),
            loading: settled(),
            error: none(),
            age_of,
            on_cancel: None,
        }
    }
}

impl DirectoryPageProps {
    /// The server's shape: the people this instance publishes.
    #[must_use]
    pub fn server(accounts: Vec<Account>) -> Self {
        Self {
            local_only: Signal::derive(|| true),
            accounts: held(accounts),
            loading: settled(),
            error: none(),
            cursor: none(),
            on_scope: None,
            on_open: None,
            on_more: None,
        }
    }
}

impl SavedPageProps {
    /// The server's shape: one shelf, its posts, and the card painter.
    #[must_use]
    pub fn server(shelf: Shelf, items: Vec<Status>, card: CardRenderer) -> Self {
        Self {
            shelf: held(shelf),
            items: held(items),
            loading: settled(),
            error: none(),
            cursor: none(),
            card,
            on_shelf: None,
            on_more: None,
        }
    }
}

impl SearchPageProps {
    /// The server's shape: the answer, the query that asked for it, and the card painter.
    #[must_use]
    pub fn server(
        query: String,
        results: SearchResults,
        age_of: Option<Callback<String, String>>,
        card: Option<Callback<Status, AnyView>>,
    ) -> Self {
        Self {
            query: held(query),
            accounts: held(results.accounts),
            statuses: held(results.statuses),
            tags: held(results.hashtags),
            searched: Signal::derive(|| true),
            loading: settled(),
            error: none(),
            on_query: None,
            on_search: None,
            age_of,
            card,
        }
    }
}
