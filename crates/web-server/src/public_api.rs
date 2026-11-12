//! Server-side reads of a public origin.
//!
//! The browser never talks to a third-party instance: a shared link is resolved by this
//! host and the bytes are handed back same-origin. A signed-in reader's own home instance
//! is asked first to resolve the link, so the home server pulls the remote copy it keeps.

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, header};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use serde_json::Value;

use crate::{App, session_of};

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
    let Some(session) = session_of(app, headers) else {
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
        .map(|max| format!("&max_id={max}"))
        .unwrap_or_default();
    relay(
        format!("https://{target}/api/v1/accounts/{id}/statuses?limit=20{suffix}"),
        token.as_deref(),
    )
    .await
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
