//! The stateless runtime the server renders through.
//!
//! It answers only the reads the request already fetched. Everything else is reported as
//! unavailable, which the shared route renderer draws as that screen's own failure state.

use std::collections::HashMap;
use std::sync::Arc;

use leptos::prelude::*;
use nodus_instance::model::{Account, Context, List, Relationship, Status};
use nodus_state::page::{PageEntry, PageKind};
use nodus_view::route::{Runtime, TimelineKind, TimelineReply};

/// One prefetched timeline page.
pub struct StaticRuntime {
    statuses: Vec<Status>,
    cursor: Option<String>,
    pages: HashMap<PageKind, PageEntry>,
    query: HashMap<String, String>,
    statuses_by_id: HashMap<String, Status>,
    contexts: HashMap<String, Context>,
    accounts: HashMap<String, Account>,
    relationships: HashMap<String, Relationship>,
    profile_statuses: Option<(Vec<Status>, Option<String>)>,
    lists: Option<Vec<List>>,
}

impl StaticRuntime {
    /// A runtime with no prefetched answers: every read is the screen's own unavailable state.
    #[must_use]
    pub fn empty() -> Arc<dyn Runtime> {
        Arc::new(Self {
            statuses: Vec::new(),
            cursor: None,
            pages: HashMap::new(),
            query: HashMap::new(),
            statuses_by_id: HashMap::new(),
            contexts: HashMap::new(),
            accounts: HashMap::new(),
            relationships: HashMap::new(),
            profile_statuses: None,
            lists: None,
        })
    }

    /// A runtime that can paint exactly this page.
    #[must_use]
    pub fn page(statuses: Vec<Status>, cursor: Option<String>) -> Arc<dyn Runtime> {
        Arc::new(Self {
            statuses,
            cursor,
            pages: HashMap::new(),
            query: HashMap::new(),
            statuses_by_id: HashMap::new(),
            contexts: HashMap::new(),
            accounts: HashMap::new(),
            relationships: HashMap::new(),
            profile_statuses: None,
            lists: None,
        })
    }

    /// A runtime that can paint the pages already fetched for this request.
    #[must_use]
    pub fn pages(pages: impl IntoIterator<Item = (PageKind, PageEntry)>) -> Arc<dyn Runtime> {
        Arc::new(Self {
            statuses: Vec::new(),
            cursor: None,
            pages: pages.into_iter().collect(),
            query: HashMap::new(),
            statuses_by_id: HashMap::new(),
            contexts: HashMap::new(),
            accounts: HashMap::new(),
            relationships: HashMap::new(),
            profile_statuses: None,
            lists: None,
        })
    }

    /// The same, with the request's query string available to routes that take one.
    #[must_use]
    pub fn pages_with_query(
        pages: impl IntoIterator<Item = (PageKind, PageEntry)>,
        query: impl IntoIterator<Item = (String, String)>,
    ) -> Arc<dyn Runtime> {
        Arc::new(Self {
            statuses: Vec::new(),
            cursor: None,
            pages: pages.into_iter().collect(),
            query: query.into_iter().collect(),
            statuses_by_id: HashMap::new(),
            contexts: HashMap::new(),
            accounts: HashMap::new(),
            relationships: HashMap::new(),
            profile_statuses: None,
            lists: None,
        })
    }

    /// One status and its conversation.
    #[must_use]
    pub fn thread(status: Status, context: Context) -> Arc<dyn Runtime> {
        let id = status.id.clone();
        Arc::new(Self {
            statuses: Vec::new(),
            cursor: None,
            pages: HashMap::new(),
            query: HashMap::new(),
            statuses_by_id: HashMap::from([(id.clone(), status)]),
            contexts: HashMap::from([(id, context)]),
            accounts: HashMap::new(),
            relationships: HashMap::new(),
            profile_statuses: None,
            lists: None,
        })
    }

    /// One account, its relationship, and the posts already fetched for it.
    #[must_use]
    pub fn profile(
        account: Account,
        relationship: Option<Relationship>,
        statuses: Vec<Status>,
        cursor: Option<String>,
    ) -> Arc<dyn Runtime> {
        Arc::new(Self {
            statuses: Vec::new(),
            cursor: None,
            pages: HashMap::new(),
            query: HashMap::new(),
            statuses_by_id: HashMap::new(),
            contexts: HashMap::new(),
            relationships: relationship
                .map(|relationship| (relationship.id.0.clone(), relationship))
                .into_iter()
                .collect(),
            accounts: HashMap::from([(account.id.clone(), account)]),
            profile_statuses: Some((statuses, cursor)),
            lists: None,
        })
    }

    /// The lists already fetched for this request.
    #[must_use]
    pub fn lists(lists: Vec<List>) -> Arc<dyn Runtime> {
        Arc::new(Self {
            statuses: Vec::new(),
            cursor: None,
            pages: HashMap::new(),
            query: HashMap::new(),
            statuses_by_id: HashMap::new(),
            contexts: HashMap::new(),
            accounts: HashMap::new(),
            relationships: HashMap::new(),
            profile_statuses: None,
            lists: Some(lists),
        })
    }
}

impl Runtime for StaticRuntime {
    fn revision(&self) -> Signal<u64> {
        Signal::derive(|| 0)
    }

    fn timeline(&self, kind: &TimelineKind) -> TimelineReply {
        let (items, cursor) = match kind {
            TimelineKind::Account(_) | TimelineKind::List(_) => self
                .profile_statuses
                .clone()
                .unwrap_or((self.statuses.clone(), self.cursor.clone())),
            _ => (self.statuses.clone(), self.cursor.clone()),
        };
        TimelineReply {
            items,
            cursor,
            held: true,
            unavailable: false,
        }
    }

    fn load_timeline(
        &self,
        _kind: TimelineKind,
        _cursor: Option<String>,
        done: Callback<Result<(), String>>,
    ) {
        done.run(Ok(()));
    }

    fn lists(&self) -> Option<Vec<List>> {
        self.lists.clone()
    }

    fn page(&self, kind: &PageKind) -> Option<PageEntry> {
        self.pages.get(kind).cloned()
    }

    fn load_page(
        &self,
        kind: PageKind,
        _cursor: Option<String>,
        done: Callback<Result<(), String>>,
    ) {
        done.run(if self.pages.contains_key(&kind) {
            Ok(())
        } else {
            Err("This page is not available without the app.".to_owned())
        });
    }

    fn status(&self, id: &str) -> Option<Status> {
        self.statuses_by_id.get(id).cloned()
    }

    fn context(&self, id: &str) -> Option<Context> {
        self.contexts.get(id).cloned()
    }

    fn account(&self, id: &str) -> Option<Account> {
        self.accounts.get(id).cloned()
    }

    fn relationship(&self, id: &str) -> Option<Relationship> {
        self.relationships.get(id).cloned()
    }

    fn load_thread(
        &self,
        id: String,
        status_done: Callback<Result<Status, String>>,
        context_done: Callback<Result<Context, String>>,
    ) {
        match (self.status(&id), self.context(&id)) {
            (Some(found_status), Some(found_context)) => {
                status_done.run(Ok(found_status));
                context_done.run(Ok(found_context));
            }
            _ => {
                let message = "This conversation is not available without the app.".to_owned();
                status_done.run(Err(message.clone()));
                context_done.run(Err(message));
            }
        }
    }

    fn load_profile(
        &self,
        id: String,
        account: Callback<Result<Account, String>>,
        relationship: Callback<Result<Option<Relationship>, String>>,
    ) {
        match self.account(&id) {
            Some(found) => {
                account.run(Ok(found));
                relationship.run(Ok(self.relationship(&id)));
            }
            None => {
                let message = "This account is not available without the app.".to_owned();
                account.run(Err(message.clone()));
                relationship.run(Err(message));
            }
        }
    }

    fn now_millis(&self) -> f64 {
        crate::rows::now_millis()
    }

    fn query(&self, key: &str) -> Option<String> {
        self.query.get(key).cloned()
    }
}
