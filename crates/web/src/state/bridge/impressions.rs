//! Reporting what the client was shown.
//!
//! An impression is the denominator of every other signal: the server shrinks a
//! rejection by `n / (n + 3)` impressions and counts `seen_count_7d` from them, so
//! without impressions a penalty never decays and fatigue never registers. The
//! cheapest honest definition is "served in a feed page": a page that arrives has been
//! rendered, and the server already stamps `recommendation_item.served_at` for the same
//! event. Viewport-level visibility (and dwell) is a later refinement that needs an
//! `IntersectionObserver`; it would narrow this definition, not replace it.

use nodus_shared::{EventBatch, EventKind, InteractionEvent};

use super::handle;

/// The server's documented maximum events per request.
const MAX_EVENTS_PER_BATCH: usize = 100;

/// Reports the items of one feed page as impressions.
///
/// Fire-and-forget on purpose: an impression is not a user action, so a failed batch
/// is dropped rather than retried or surfaced. Retrying would compete with the
/// request that actually produced the page, and the loss is one observation in a
/// counter that already tolerates gaps.
pub fn report_impressions(feed_id: &str, item_ids: &[i64]) {
    let Some(sdk) = handle() else {
        return;
    };
    let client_ts: String = js_sys::Date::new_0().to_iso_string().into();
    for chunk in item_ids.chunks(MAX_EVENTS_PER_BATCH) {
        let batch = EventBatch {
            feed_id: feed_id.to_owned(),
            events: chunk
                .iter()
                .map(|item_id| InteractionEvent {
                    item_id: *item_id,
                    kind: EventKind::Impression,
                    client_ts: client_ts.clone(),
                    dwell_ms: None,
                })
                .collect(),
        };
        let sdk = sdk.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let _ = sdk.report_events(batch).await;
        });
    }
}
