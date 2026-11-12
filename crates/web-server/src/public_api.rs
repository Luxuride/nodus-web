//! Server-side reads of a public origin.
//!
//! The browser never talks to a third-party instance: a shared link is resolved by this
//! host and the bytes are handed back same-origin. That keeps CORS, certificates, and
//! instance-specific API quirks out of the client.

use axum::extract::{Path, Query};
use axum::http::header;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

fn host_valid(host: &str) -> bool {
    !host.is_empty()
        && host
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | ':'))
        && host.contains('.')
}

async fn relay(url: String) -> Response {
    let response = match reqwest::Client::new().get(&url).send().await {
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
    (
        status,
        [(header::CONTENT_TYPE, "application/json")],
        body,
    )
        .into_response()
}

pub async fn status(Path((host, id)): Path<(String, String)>) -> Response {
    if !host_valid(&host) {
        return axum::http::StatusCode::BAD_REQUEST.into_response();
    }
    relay(format!("https://{host}/api/v1/statuses/{id}")).await
}

pub async fn context(Path((host, id)): Path<(String, String)>) -> Response {
    if !host_valid(&host) {
        return axum::http::StatusCode::BAD_REQUEST.into_response();
    }
    relay(format!("https://{host}/api/v1/statuses/{id}/context")).await
}

pub async fn account(Path((host, id)): Path<(String, String)>) -> Response {
    if !host_valid(&host) {
        return axum::http::StatusCode::BAD_REQUEST.into_response();
    }
    relay(format!("https://{host}/api/v1/accounts/{id}")).await
}

#[derive(Debug, Deserialize)]
pub struct Before {
    max_id: Option<String>,
}

pub async fn account_statuses(
    Path((host, id)): Path<(String, String)>,
    Query(before): Query<Before>,
) -> Response {
    if !host_valid(&host) {
        return axum::http::StatusCode::BAD_REQUEST.into_response();
    }
    let suffix = before
        .max_id
        .map(|max| format!("&max_id={max}"))
        .unwrap_or_default();
    relay(format!(
        "https://{host}/api/v1/accounts/{id}/statuses?limit=20{suffix}"
    ))
    .await
}

pub async fn followers(Path((host, id)): Path<(String, String)>) -> Response {
    if !host_valid(&host) {
        return axum::http::StatusCode::BAD_REQUEST.into_response();
    }
    relay(format!(
        "https://{host}/api/v1/accounts/{id}/followers?limit=40"
    ))
    .await
}

pub async fn following(Path((host, id)): Path<(String, String)>) -> Response {
    if !host_valid(&host) {
        return axum::http::StatusCode::BAD_REQUEST.into_response();
    }
    relay(format!(
        "https://{host}/api/v1/accounts/{id}/following?limit=40"
    ))
    .await
}
