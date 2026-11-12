//! Server-side reads of a public origin.
//!
//! The browser never talks to a third-party instance: a shared link is resolved by this
//! host and the bytes are handed back same-origin. A signed-in reader's own home instance
//! is asked first to resolve the link, so the home server pulls the remote copy it keeps.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use serde_json::Value;

use crate::{App, session_of};

fn local_host() -> Option<String> {
    std::env::var("NODUS_INSTANCE_HOST").ok()
}

fn hydrate_base() -> Option<String> {
    std::env::var("NODUS_HYDRATE_URL").ok()
}

async fn hydrate(path: &str) -> Option<Value> {
    let base = hydrate_base()?;
    let mut request = reqwest::Client::new()
        .get(format!("{}{path}", base.trim_end_matches('/')))
        .timeout(std::time::Duration::from_secs(8));
    if let Ok(token) = std::env::var("NODUS_HYDRATE_TOKEN") {
        request = request.bearer_auth(token);
    }
    let response = request.send().await.ok()?;
    if !response.status().is_success() {
        return None;
    }
    response.json().await.ok()
}

fn hydrated_account(account: &Value) -> Value {
    let acct = account
        .get("acct")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let username = acct.split('@').next().unwrap_or(acct);
    serde_json::json!({
        "id": account.get("id").cloned().unwrap_or(Value::Null),
        "username": username,
        "acct": acct,
        "display_name": account.get("display_name").cloned().unwrap_or(Value::Null),
        "note": account.get("note").cloned().unwrap_or(Value::Null),
        "url": account.get("url").cloned().unwrap_or(Value::Null),
        "avatar": account.get("avatar").cloned().unwrap_or(Value::Null),
        "header": account.get("header").cloned().unwrap_or(Value::Null),
        "followers_count": account.get("followers_count").cloned().unwrap_or(Value::Null),
        "following_count": account.get("following_count").cloned().unwrap_or(Value::Null),
        "statuses_count": 0,
        "bot": false,
        "locked": false,
        "created_at": "",
        "emojis": [],
        "fields": []
    })
}

fn hydrated_status(post: &Value, account: &Value) -> Value {
    let media: Vec<Value> = post
        .get("media")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .map(|media| {
                    serde_json::json!({
                        "id": media.get("url").and_then(Value::as_str).unwrap_or_default(),
                        "type": media.get("kind").and_then(Value::as_str).unwrap_or("unknown"),
                        "url": media.get("url").cloned().unwrap_or(Value::Null),
                        "preview_url": media.get("preview_url").cloned().unwrap_or(Value::Null),
                        "description": media.get("description").cloned().unwrap_or(Value::Null),
                        "meta": {
                            "original": {
                                "width": media.get("width").cloned().unwrap_or(Value::Null),
                                "height": media.get("height").cloned().unwrap_or(Value::Null)
                            }
                        }
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    serde_json::json!({
        "id": post.get("id").cloned().unwrap_or(Value::Null),
        "created_at": post.get("published").cloned().unwrap_or(Value::Null),
        "content": post.get("content").cloned().unwrap_or(Value::Null),
        "url": post.get("url").cloned().unwrap_or(Value::Null),
        "visibility": "public",
        "sensitive": post.get("sensitive").cloned().unwrap_or(Value::Bool(false)),
        "spoiler_text": post.get("spoiler_text").cloned().unwrap_or(Value::Null),
        "favourites_count": 0,
        "reblogs_count": 0,
        "replies_count": post.get("replies_count").cloned().unwrap_or(Value::Null),
        "favourited": false,
        "reblogged": false,
        "bookmarked": false,
        "reblog": null,
        "in_reply_to_id": null,
        "media_attachments": media,
        "emojis": post.get("emojis").cloned().unwrap_or(Value::Array(Vec::new())),
        "mentions": [],
        "tags": [],
        "card": null,
        "poll": null,
        "edited_at": null,
        "account": hydrated_account(account)
    })
}

fn host_valid(host: &str) -> bool {
    !host.is_empty()
        && host
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | ':'))
        && host.contains('.')
}

async fn json(url: &str, token: Option<&str>) -> Option<Value> {
    let mut request = reqwest::Client::new().get(url);
    if let Some(token) = token {
        request = request.bearer_auth(token);
    }
    let response = request.send().await.ok()?;
    if !response.status().is_success() {
        return None;
    }
    response.json().await.ok()
}

async fn relay(url: String, token: Option<&str>) -> Response {
    let mut request = reqwest::Client::new().get(&url);
    if let Some(token) = token {
        request = request.bearer_auth(token);
    }
    let response = match request.send().await {
        Ok(response) => response,
        Err(error) => {
            return (axum::http::StatusCode::BAD_GATEWAY, error.to_string()).into_response();
        }
    };
    let status = response.status();
    let body = match response.text().await {
        Ok(body) => body,
        Err(error) => {
            return (axum::http::StatusCode::BAD_GATEWAY, error.to_string()).into_response();
        }
    };
    (status, [(header::CONTENT_TYPE, "application/json")], body).into_response()
}

/// Resolve an origin account through the reader's home instance.
async fn account_target(
    app: &App,
    headers: &HeaderMap,
    host: &str,
    id: &str,
) -> (String, String, Option<String>) {
    let session = session_of(app, headers);
    if session.is_none()
        && local_host().as_deref() == Some(host)
        && let Some(local) = json(&format!("https://{host}/api/v1/accounts/{id}"), None).await
        && let Some(acct) = local.get("acct").and_then(Value::as_str)
        && acct.contains('@')
        && let Some(origin) = acct.rsplit('@').next()
        && host_valid(origin)
    {
        let resolved = hydrate(&format!("/v1/ap/account?acct={}", urlencoding(acct))).await;
        if let Some(remote) = resolved
            .as_ref()
            .and_then(|value| value.get("id"))
            .and_then(Value::as_str)
        {
            return (origin.to_owned(), remote.to_owned(), None);
        }
    }
    let Some(session) = session else {
        return (host.to_owned(), id.to_owned(), None);
    };
    if session.host == host {
        return (host.to_owned(), id.to_owned(), Some(session.token));
    }
    let origin = json(&format!("https://{host}/api/v1/accounts/{id}"), None).await;
    let Some(url) = origin
        .as_ref()
        .and_then(|value| value.get("url"))
        .and_then(Value::as_str)
    else {
        return (host.to_owned(), id.to_owned(), None);
    };
    let query = urlencoding(url);
    let search = json(
        &format!(
            "https://{}/api/v2/search?q={query}&resolve=true&type=accounts&limit=1",
            session.host
        ),
        Some(&session.token),
    )
    .await;
    let local = search
        .as_ref()
        .and_then(|value| value.get("accounts"))
        .and_then(Value::as_array)
        .and_then(|accounts| accounts.first())
        .and_then(|account| account.get("id"))
        .and_then(Value::as_str);
    match local {
        Some(local) => (session.host.clone(), local.to_owned(), Some(session.token)),
        None => (host.to_owned(), id.to_owned(), None),
    }
}

fn urlencoding(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(char::from(byte));
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

/// The same resolution for a status, using its origin URL.
async fn status_target(
    app: &App,
    headers: &HeaderMap,
    host: &str,
    id: &str,
) -> (String, String, Option<String>) {
    let Some(session) = session_of(app, headers) else {
        return (host.to_owned(), id.to_owned(), None);
    };
    if session.host == host {
        return (host.to_owned(), id.to_owned(), Some(session.token));
    }
    let origin = json(&format!("https://{host}/api/v1/statuses/{id}"), None).await;
    let Some(url) = origin
        .as_ref()
        .and_then(|value| value.get("url"))
        .and_then(Value::as_str)
    else {
        return (host.to_owned(), id.to_owned(), None);
    };
    let query = urlencoding(url);
    let search = json(
        &format!(
            "https://{}/api/v2/search?q={query}&resolve=true&type=statuses&limit=1",
            session.host
        ),
        Some(&session.token),
    )
    .await;
    let local = search
        .as_ref()
        .and_then(|value| value.get("statuses"))
        .and_then(Value::as_array)
        .and_then(|statuses| statuses.first())
        .and_then(|status| status.get("id"))
        .and_then(Value::as_str);
    match local {
        Some(local) => (session.host.clone(), local.to_owned(), Some(session.token)),
        None => (host.to_owned(), id.to_owned(), None),
    }
}

pub async fn status(
    State(app): State<Arc<App>>,
    headers: HeaderMap,
    Path((host, id)): Path<(String, String)>,
) -> Response {
    if !host_valid(&host) {
        return axum::http::StatusCode::BAD_REQUEST.into_response();
    }
    let (target, id, token) = status_target(&app, &headers, &host, &id).await;
    relay(
        format!("https://{target}/api/v1/statuses/{id}"),
        token.as_deref(),
    )
    .await
}

pub async fn context(
    State(app): State<Arc<App>>,
    headers: HeaderMap,
    Path((host, id)): Path<(String, String)>,
) -> Response {
    if !host_valid(&host) {
        return axum::http::StatusCode::BAD_REQUEST.into_response();
    }
    let (target, id, token) = status_target(&app, &headers, &host, &id).await;
    relay(
        format!("https://{target}/api/v1/statuses/{id}/context"),
        token.as_deref(),
    )
    .await
}

pub async fn account(
    State(app): State<Arc<App>>,
    headers: HeaderMap,
    Path((host, id)): Path<(String, String)>,
) -> Response {
    if !host_valid(&host) {
        return axum::http::StatusCode::BAD_REQUEST.into_response();
    }
    let (target, id, token) = account_target(&app, &headers, &host, &id).await;
    relay(
        format!("https://{target}/api/v1/accounts/{id}"),
        token.as_deref(),
    )
    .await
}

#[derive(Debug, Deserialize)]
pub struct Before {
    max_id: Option<String>,
}

pub async fn account_statuses(
    State(app): State<Arc<App>>,
    headers: HeaderMap,
    Path((host, id)): Path<(String, String)>,
    Query(before): Query<Before>,
) -> Response {
    if !host_valid(&host) {
        return axum::http::StatusCode::BAD_REQUEST.into_response();
    }
    let (target, id, token) = account_target(&app, &headers, &host, &id).await;
    let suffix = before
        .max_id
        .as_ref()
        .map(|max| format!("?max_id={max}"))
        .unwrap_or_default();
    if let Some(value) = json(
        &format!(
            "https://{target}/api/v1/accounts/{id}/statuses?limit=20{}",
            suffix.replace('?', "&")
        ),
        token.as_deref(),
    )
    .await
        && value.as_array().is_some_and(|items| !items.is_empty())
    {
        return Json(value).into_response();
    }
    let account = hydrate(&format!("/v1/account/{target}/{id}")).await;
    let posts = hydrate(&format!("/v1/account/{target}/{id}/statuses{suffix}")).await;
    if let (Some(account), Some(posts)) = (account, posts) {
        let items: Vec<Value> = posts
            .get("items")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .map(|post| hydrated_status(post, &account))
                    .collect()
            })
            .unwrap_or_default();
        return Json(Value::Array(items)).into_response();
    }
    (StatusCode::BAD_GATEWAY, "account statuses unavailable").into_response()
}

pub async fn followers(
    State(app): State<Arc<App>>,
    headers: HeaderMap,
    Path((host, id)): Path<(String, String)>,
) -> Response {
    if !host_valid(&host) {
        return axum::http::StatusCode::BAD_REQUEST.into_response();
    }
    let (target, id, token) = account_target(&app, &headers, &host, &id).await;
    relay(
        format!("https://{target}/api/v1/accounts/{id}/followers?limit=40"),
        token.as_deref(),
    )
    .await
}

pub async fn following(
    State(app): State<Arc<App>>,
    headers: HeaderMap,
    Path((host, id)): Path<(String, String)>,
) -> Response {
    if !host_valid(&host) {
        return axum::http::StatusCode::BAD_REQUEST.into_response();
    }
    let (target, id, token) = account_target(&app, &headers, &host, &id).await;
    relay(
        format!("https://{target}/api/v1/accounts/{id}/following?limit=40"),
        token.as_deref(),
    )
    .await
}
