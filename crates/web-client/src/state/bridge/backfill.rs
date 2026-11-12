//! Ask the recommender to crawl what the reader just opened.

use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;

use crate::state::bridge::{session_token, with_sdk};

/// Records a status or profile visit as backfill work.
pub fn request(kind: &'static str, host: String, id: String, acct: Option<String>) {
    let body = serde_json::json!({
        "kind": kind,
        "host": host,
        "id": id,
        "acct": acct,
    });
    let Ok(request) = Request::post("/api/v1/browse")
        .header("content-type", "application/json")
        .body(body.to_string())
    else {
        return;
    };
    spawn_local(async move {
        let _ = request.send().await;
    });
}

/// Ask the reader's own Mastodon server to pull this URL over ActivityPub.
///
/// `resolve=true` is the one public API that makes the home instance fetch a remote
/// account or post it has not seen. It does not backfill an entire outbox, but it makes
/// the next read of the profile or thread local.
pub fn hydrate(url: String) {
    let Some(host) = with_sdk(|sdk| sdk.state().session.get().map(|session| session.host)) else {
        return;
    };
    let Some(token) = session_token() else {
        return;
    };
    let query = encode(&url);
    let request = Request::get(&format!(
        "https://{host}/api/v2/search?q={query}&resolve=true&limit=1"
    ))
    .header("authorization", &format!("Bearer {token}"));
    spawn_local(async move {
        let _ = request.send().await;
    });
}

fn encode(value: &str) -> String {
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
