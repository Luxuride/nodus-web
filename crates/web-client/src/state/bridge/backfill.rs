//! Ask the recommender to crawl what the reader just opened.

use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;

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
