//! Reading a post without a session.
//!
//! A shared link has to work for someone who has never signed in — that is the point of
//! sharing it. The browser asks our own host, which performs the one public read of the
//! origin and hands back the bytes. That keeps third-party origins, CORS and per-instance
//! API quirks out of the client; the client only ever talks to Nodus.

use gloo_net::http::Request;
use nodus_client_core::model::{Account, Context, Status};

/// Why a public read did not produce a post.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PublicError {
    /// The host is not a bare host name, so it cannot be put in a URL.
    BadHost,
    /// The instance answered, but not with this post — usually "not federated here".
    NotFound,
    /// The instance does not serve public posts to anonymous readers.
    Forbidden,
    /// Anything else, with the status code when there was one.
    Failed(String),
}

impl PublicError {
    /// A sentence a reader can act on.
    pub fn message(&self) -> String {
        match self {
            Self::BadHost => "That link does not name an instance.".to_owned(),
            Self::NotFound => {
                "This post is not available on that instance. It may be private, deleted, or not federated there."
                    .to_owned()
            }
            Self::Forbidden => "That instance does not show posts to visitors.".to_owned(),
            Self::Failed(detail) => format!("The instance could not be reached: {detail}"),
        }
    }
}

fn url(host: &str, path: &str) -> Result<String, PublicError> {
    // A host is a host, not a URL: no scheme, no path, no whitespace. Anything else would
    // let a link smuggle a different origin into a fetch.
    let valid = !host.is_empty()
        && host
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | ':'))
        && host.contains('.');
    if !valid {
        return Err(PublicError::BadHost);
    }
    Ok(format!("/public-api{path}"))
}

async fn get<T: serde::de::DeserializeOwned>(host: &str, path: &str) -> Result<T, PublicError> {
    let url = url(host, path)?;
    let response = Request::get(&url)
        .send()
        .await
        .map_err(|error| PublicError::Failed(error.to_string()))?;
    match response.status() {
        200..=299 => {}
        401 | 403 => return Err(PublicError::Forbidden),
        404 | 410 => return Err(PublicError::NotFound),
        status => return Err(PublicError::Failed(format!("HTTP {status}"))),
    }
    let body = response
        .text()
        .await
        .map_err(|error| PublicError::Failed(error.to_string()))?;
    serde_json::from_str(&body).map_err(|error| PublicError::Failed(error.to_string()))
}

/// The post itself.
pub async fn status(host: &str, id: &str) -> Result<Status, PublicError> {
    get(host, &format!("/status/{host}/{id}")).await
}

/// The replies above and below it.
pub async fn context(host: &str, id: &str) -> Result<Context, PublicError> {
    get(host, &format!("/status/{host}/{id}/context")).await
}

/// The account behind a shared link.
pub async fn account(host: &str, id: &str) -> Result<Account, PublicError> {
    get(host, &format!("/account/{host}/{id}")).await
}

/// The account's public posts, newest first.
pub async fn account_statuses(host: &str, id: &str) -> Result<Vec<Status>, PublicError> {
    get(host, &format!("/account/{host}/{id}/statuses")).await
}

/// The account's public posts older than a status id.
pub async fn account_statuses_before(
    host: &str,
    id: &str,
    max_id: &str,
) -> Result<Vec<Status>, PublicError> {
    get(
        host,
        &format!("/account/{host}/{id}/statuses?max_id={max_id}"),
    )
    .await
}

/// The account's public followers.
pub async fn account_followers(host: &str, id: &str) -> Result<Vec<Account>, PublicError> {
    get(host, &format!("/account/{host}/{id}/followers")).await
}

/// The accounts this account follows publicly.
pub async fn account_following(host: &str, id: &str) -> Result<Vec<Account>, PublicError> {
    get(host, &format!("/account/{host}/{id}/following")).await
}
