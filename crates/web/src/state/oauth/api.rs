//! The two requests the flow makes to the instance's own API: app registration and
//! the code-to-token exchange.
//!
//! Both are form-encoded `POST`s through `gloo-net` (already in the graph via the
//! transports), so the browser's `fetch` and CORS rules apply unchanged. JSON is
//! parsed with the browser's own parser: `serde_json` is not a dependency of this
//! crate, and adding it for two scalars would be a larger change than the feature.

use gloo_net::http::{Headers, Request};
use wasm_bindgen::JsValue;

use super::SCOPES;
use super::pending::Pending;

/// Name the instance shows on its "authorized applications" page.
const CLIENT_NAME: &str = "nodus";

/// How much of a non-JSON error body is quoted back to the user. Enough to recognise
/// a reverse-proxy error page, short enough for an inline alert.
const MAX_ERROR_CHARS: usize = 200;

/// Registers the app with `host` and returns its public `client_id`.
pub(super) async fn register_app(
    host: &str,
    origin: &str,
    redirect_uri: &str,
) -> Result<String, String> {
    let url = endpoint(host, "/api/v1/apps");
    let json = post_form(
        &url,
        &[
            ("client_name", CLIENT_NAME),
            ("redirect_uris", redirect_uri),
            ("scopes", SCOPES),
            ("website", origin),
        ],
    )
    .await?;
    // `client_secret` is ignored on purpose: PKCE makes this a public client, and a
    // secret in browser storage is a secret one `localStorage` read away.
    string_field(&json, "client_id")
        .filter(|id| !id.is_empty())
        .ok_or_else(|| {
            format!("{url} returned no client id. The instance may have app registration disabled.")
        })
}

/// Exchanges an authorization code for an access token.
///
/// The verifier proves this browser is the one that started the flow; `redirect_uri`
/// must repeat the registered value byte-for-byte, which is why it is stored rather
/// than rebuilt.
pub(super) async fn exchange_code(record: &Pending, code: &str) -> Result<String, String> {
    let url = endpoint(&record.host, "/oauth/token");
    let json = post_form(
        &url,
        &[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("client_id", record.client_id.as_str()),
            ("redirect_uri", record.redirect_uri.as_str()),
            ("code_verifier", record.verifier.as_str()),
            ("scope", SCOPES),
        ],
    )
    .await?;
    string_field(&json, "access_token")
        .filter(|token| !token.is_empty())
        .ok_or_else(|| format!("{url} returned no access token."))
}

/// `https://` is hard-coded: the client id, the code and the token travel on these
/// requests, and Mastodon's OAuth endpoints are https-only in practice. An instance
/// reachable only over plain http is what the token fallback is for.
pub(super) fn endpoint(host: &str, path: &str) -> String {
    format!("https://{host}{path}")
}

/// `POST`s a form-encoded body and returns the parsed JSON response.
///
/// A non-2xx response becomes an `Err` carrying the instance's own reason, which is
/// the only part of a failed exchange a user can act on.
async fn post_form(url: &str, form: &[(&str, &str)]) -> Result<JsValue, String> {
    let body = url::form_urlencoded::Serializer::new(String::new())
        .extend_pairs(form.iter().copied())
        .finish();
    let headers = Headers::new();
    headers.set("Content-Type", "application/x-www-form-urlencoded");

    let request = Request::post(url)
        .headers(headers)
        .body(body)
        .map_err(|error| format!("Could not build the request to {url}: {error}"))?;
    let response = request
        .send()
        .await
        .map_err(|error| format!("Could not reach {url}: {error}"))?;
    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|error| format!("Reading the response from {url} failed: {error}"))?;

    let json = js_sys::JSON::parse(&text).unwrap_or(JsValue::UNDEFINED);
    if !(200..300).contains(&status) {
        // Mastodon answers a refused exchange with
        // `{"error": "invalid_grant", "error_description": "…"}`; a proxy in front of
        // it may answer with HTML, which is still more useful than a bare status.
        let reason = string_field(&json, "error_description")
            .or_else(|| string_field(&json, "error"))
            .unwrap_or_else(|| text.chars().take(MAX_ERROR_CHARS).collect());
        return Err(format!("{url} responded {status}: {reason}"));
    }
    if json.is_undefined() {
        return Err(format!("{url} returned a body that is not JSON."));
    }
    Ok(json)
}

/// A string property of a JSON object, if it is there and is a string.
fn string_field(value: &JsValue, name: &str) -> Option<String> {
    js_sys::Reflect::get(value, &JsValue::from_str(name))
        .ok()?
        .as_string()
}
