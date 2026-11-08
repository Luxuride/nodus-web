//! Everything this server asks an instance: who a token belongs to, and the first page.
//!
//! Both calls go through the SDK's native client, so the endpoints, the pagination header
//! (`Link: rel="next"`) and the payload shapes are the ones the browser app already uses.
//! A second hand-written HTTP layer would be a second place to be wrong about Mastodon.

use nodus_instance::actions::{Browse, ScheduleActions, Search as SearchApi};
use nodus_instance::model::{
    Account, Context, Conversation, List, Notification, PageParams, ScheduledStatus,
    SearchResults, Status,
};
use nodus_instance::LocalId;
use nodus_instance::{HttpInstance, Instance, InstanceApi};
use nodus_runtime::reads::{Headless, Reads};

/// A signed-in instance, ready to ask.
#[derive(Debug, Clone)]
pub struct Client {
    instance: HttpInstance,
}

impl Client {
    /// Build a client for `host` with `token`. The host is whatever the reader typed in
    /// the app's sign-in screen; `Instance::parse` supplies `https://` when they omitted it.
    pub fn new(host: &str, token: &str) -> Result<Self, String> {
        let instance = Instance::parse(host).map_err(|error| error.to_string())?;
        Ok(Self {
            instance: HttpInstance::new(&instance, token.to_owned()),
        })
    }

    /// Prove the token and learn which account it belongs to.
    pub async fn whoami(&self) -> Result<Account, String> {
        self.instance
            .verify_credentials()
            .await
            .map_err(|error| error.to_string())
    }

    /// The first page of the home timeline, newest first.
    ///
    /// Read through the SDK's `Reads` trait rather than by calling the instance directly:
    /// the browser's `Sdk` implements the same trait, so the page this server paints and
    /// the page the browser would have fetched come from one definition of "home timeline".
    pub async fn home(&self, limit: u16) -> Result<(Vec<Status>, Option<String>), String> {
        let headless = Headless::new(self.instance.clone());
        let page = headless
            .home(PageParams::first(limit))
            .await
            .map_err(|error| error.to_string())?;
        Ok((page.items, page.next_max_id))
    }

    /// The local or federated timeline. Both are public reads: the cookie supplies the
    /// token, but a stranger's timeline would answer the same way.
    pub async fn feed(
        &self,
        local: bool,
        limit: u16,
    ) -> Result<(Vec<Status>, Option<String>), String> {
        let kind = if local {
            nodus_instance::StreamKind::Local
        } else {
            nodus_instance::StreamKind::Federated
        };
        let page = self
            .instance
            .timeline(kind, PageParams::first(limit))
            .await
            .map_err(|error| error.to_string())?;
        Ok((page.items, page.next_max_id))
    }

    /// Notifications, in the instance's order.
    pub async fn notifications(&self, limit: u16) -> Result<Vec<Notification>, String> {
        let page = self
            .instance
            .notifications(PageParams::first(limit))
            .await
            .map_err(|error| error.to_string())?;
        Ok(page.items)
    }

    /// Conversations, newest first.
    pub async fn conversations(&self, limit: u16) -> Result<Vec<Conversation>, String> {
        let page = self
            .instance
            .conversations(PageParams::first(limit))
            .await
            .map_err(|error| error.to_string())?;
        Ok(page.items)
    }

    /// Favourites or bookmarks.
    pub async fn saved(
        &self,
        bookmarks: bool,
        limit: u16,
    ) -> Result<(Vec<Status>, Option<String>), String> {
        let params = PageParams::first(limit);
        let page = if bookmarks {
            self.instance.bookmarks(params).await
        } else {
            self.instance.favourites(params).await
        }
        .map_err(|error| error.to_string())?;
        Ok((page.items, page.next_max_id))
    }

    /// One account, by id.
    pub async fn account(&self, id: &str) -> Result<Account, String> {
        self.instance
            .account(&LocalId(id.to_owned()))
            .await
            .map_err(|error| error.to_string())
    }

    /// One account's posts.
    pub async fn account_statuses(
        &self,
        id: &str,
        limit: u16,
    ) -> Result<(Vec<Status>, Option<String>), String> {
        let page = self
            .instance
            .account_statuses(&LocalId(id.to_owned()), PageParams::first(limit))
            .await
            .map_err(|error| error.to_string())?;
        Ok((page.items, page.next_max_id))
    }

    /// One status, and the thread around it.
    pub async fn status(&self, id: &str) -> Result<Status, String> {
        self.instance
            .status(id)
            .await
            .map_err(|error| error.to_string())
    }

    /// The thread around one status.
    pub async fn context(&self, id: &str) -> Result<Context, String> {
        self.instance
            .status_context(id)
            .await
            .map_err(|error| error.to_string())
    }

    /// The reader's lists.
    pub async fn lists(&self) -> Result<Vec<List>, String> {
        self.instance
            .lists()
            .await
            .map_err(|error| error.to_string())
    }

    /// The queue of posts waiting to go out.
    pub async fn scheduled(&self) -> Result<Vec<ScheduledStatus>, String> {
        self.instance
            .scheduled()
            .await
            .map_err(|error| error.to_string())
    }

    /// The instance's directory: people it suggests.
    pub async fn directory(&self, local: bool) -> Result<Vec<Account>, String> {
        self.instance
            .directory(local, 0)
            .await
            .map_err(|error| error.to_string())
    }

    /// A search.
    pub async fn search(&self, query: &str) -> Result<SearchResults, String> {
        self.instance
            .search(query, false)
            .await
            .map_err(|error| error.to_string())
    }
}
