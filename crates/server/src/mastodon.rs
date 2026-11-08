//! Everything this server asks an instance: who a token belongs to, and the first page.
//!
//! Both calls go through the SDK's native client, so the endpoints, the pagination header
//! (`Link: rel="next"`) and the payload shapes are the ones the browser app already uses.
//! A second hand-written HTTP layer would be a second place to be wrong about Mastodon.

use nodus_instance::model::{Account, PageParams, Status, StreamKind};
use nodus_instance::{HttpInstance, Instance, InstanceApi};

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
    pub async fn home(&self, limit: u16) -> Result<(Vec<Status>, Option<String>), String> {
        let page = self
            .instance
            .timeline(StreamKind::User, PageParams::first(limit))
            .await
            .map_err(|error| error.to_string())?;
        Ok((page.items, page.next_max_id))
    }
}
