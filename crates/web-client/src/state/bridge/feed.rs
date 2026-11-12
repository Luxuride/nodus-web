//! The Feed tab's verbs: loading pages and acting on a single card.
//!
//! Everything here changes or observes the SDK through `state::bridge`, so views
//! stay declarative; this module is where an action turns into a request plus a
//! command. `state::bridge` re-exports the public verbs, so call sites keep saying
//! `bridge::follow_item`.

use leptos::prelude::*;
use nodus_client_sdk::instance_api::LocalId;
use nodus_client_sdk::{Command, Degraded, InstanceApi};
use nodus_shared::{AccountFeed, EventBatch, EventKind, InteractionEvent};

use super::{
    FEED_ID, FeedUi, WebSdk, dispatch, existing_feed_id, feed_ui, forget_feed_id, handle,
    impressions, instance_transport, with_sdk,
};

/// Loads one page of the feed, keeping the current list visible while it runs.
///
/// `cursor` is `None` for a refresh and the previous page's `next_cursor` for
/// "Load more".
pub fn reload_feed(cursor: Option<String>) {
    let Some(sdk) = handle() else {
        return;
    };
    let ui = feed_ui();
    ui.refreshing.set(true);
    ui.error.set(None);
    // A refresh names the snapshot it is leaving behind, so the server ranks again
    // instead of handing the same list back inside its reuse window; "Load more" is a
    // read of that snapshot and must not be mistaken for one.
    let superseding = cursor.is_none().then(existing_feed_id).flatten();
    wasm_bindgen_futures::spawn_local(async move {
        let result = match superseding {
            Some(feed_id) => sdk.refresh_feed(&feed_id).await,
            None => sdk.load_feed(cursor.as_deref()).await,
        };
        match result {
            Ok(feed) => remember_feed(&feed),
            // A restarted discovery process forgets its snapshots. The id names
            // nothing now, so retry once as a fresh read instead of making every
            // refresh repeat the same dead request.
            Err(error) if gone(&error) => {
                forget_feed_id();
                match sdk.load_feed(None).await {
                    Ok(feed) => remember_feed(&feed),
                    Err(error) => ui.error.set(Some(error.to_string())),
                }
            }
            Err(error) => ui.error.set(Some(error.to_string())),
        }
        ui.refreshing.set(false);
    });
}

fn gone(
    error: &nodus_client_sdk::SdkError<nodus_client_sdk::instance_api::Error, nodus_discovery::Error>,
) -> bool {
    matches!(
        error,
        nodus_client_sdk::SdkError::Discovery(nodus_discovery::Error::Status { status: 410, .. })
    )
}

/// Follows the account behind a feed item and hides the item.
///
/// The follow is an instance write, not an SDK command, so it goes through the
/// transport; hiding the item is state, so it goes through the SDK. On error the
/// item stays and the caller shows the message (the button returns to "Follow").
///
/// # Errors
///
/// Returns a message fit for inline display; nothing is dismissed when the
/// instance call fails.
pub async fn follow_item(item_id: i64, account_id: &str) -> Result<(), String> {
    let transport = instance_transport().ok_or_else(|| "Not signed in.".to_owned())?;
    transport
        .follow(&LocalId(account_id.to_owned()))
        .await
        .map_err(|error| error.to_string())?;
    dispatch(Command::DismissItem { item_id });
    Ok(())
}

/// Undoes a follow from its feed card, optimistically.
///
/// A follow is one click, so a misclick must be one click to undo without opening
/// the instance. The card flips to "Follow" *before* the request because the SDK
/// cannot express a single-item follow change (see [`set_following`]); on error it
/// flips back, so the button never disagrees with the server.
///
/// Unlike [`follow_item`] this returns nothing and reports failure through
/// [`feed_ui`]: the flip changes the card's list key, so the card remounts and any
/// error held inside it would be thrown away before it could be read. The Feed tab's
/// alert outlives the remount, which is why the message goes there.
pub async fn unfollow_item(account: String) {
    let ui = feed_ui();
    let Some(transport) = instance_transport() else {
        ui.error.set(Some("Not signed in.".to_owned()));
        return;
    };
    ui.error.set(None);
    set_following(&account, false);
    if let Err(error) = transport.unfollow(&LocalId(account.clone())).await {
        set_following(&account, true);
        ui.error.set(Some(error.to_string()));
    }
}

/// Reports "not interested" for a feed item, then hides the item.
///
/// # Errors
///
/// Returns a message fit for inline display; the item is kept when the report
/// fails, so the user's action is never silently lost.
pub async fn not_interested(item_id: i64) -> Result<(), String> {
    let sdk = handle().ok_or_else(|| "Not signed in.".to_owned())?;
    let feed_id = FEED_ID
        .with_borrow(Clone::clone)
        .ok_or_else(|| "The feed has not loaded yet.".to_owned())?;

    let batch = EventBatch {
        feed_id,
        events: vec![InteractionEvent {
            item_id,
            kind: EventKind::NotInterested,
            client_ts: client_timestamp(),
            dwell_ms: None,
        }],
    };
    sdk.report_events(batch)
        .await
        .map_err(|error| error.to_string())?;
    dispatch(Command::DismissItem { item_id });
    Ok(())
}

/// Rewrites the held feed so one account's follow flag reads `following`.
///
/// # Why this is a rebuild
///
/// The SDK's `Command` vocabulary has no "set follow" variant — only whole-feed
/// loads and dismissals — so the only way to express the change is to rebuild the
/// `AccountFeed` and dispatch `FeedLoaded`. That is a gap worth closing: a follow
/// flag is state, and a state flag should not need a fake page load.
///
/// Two details keep the fake load from doing damage:
///
/// * it rebuilds from `FeedState::visible()`, because `FeedState::replace` clears
///   `dismissed` — rebuilding from `items` would resurrect every card the user
///   already dismissed;
/// * the envelope fields (`contract`, `model_version`, `similarity_age_hours`,
///   `signals_unavailable`) are placeholders: `replace` ignores them, and a follow
///   flag is not a new server response. `feed_id`, cursor and generation time are
///   carried over, since those *are* read back.
fn set_following(account: &str, following: bool) {
    let rebuilt = with_sdk(|sdk| -> Option<AccountFeed> {
        let state = sdk.state().feed.get();
        let feed_id = FEED_ID.with_borrow(Clone::clone)?;
        // `remember_feed` sets the snapshot id and the generation time from the same
        // response, so a loaded feed has both; without a timestamp there is nothing
        // to rebuild, and the flag is left alone rather than faked.
        let generated_at = state.generated_at.clone()?;
        let items = state
            .visible()
            .cloned()
            .map(|mut item| {
                if item.account.id == account {
                    item.account.already_following = following;
                }
                item
            })
            .collect();
        Some(AccountFeed {
            feed_id,
            generated_at,
            contract: String::new(),
            model_version: 0,
            similarity_age_hours: 0.0,
            // `AccountFeed` carries one flag; `FeedState` keeps a list, so the
            // first is the one it parsed from the server and the one to keep.
            degraded: state.degraded.first().copied().map(degraded_wire),
            signals_unavailable: Vec::new(),
            items,
            next_cursor: state.next_cursor.clone(),
        })
    });
    if let Some(feed) = rebuilt {
        dispatch(Command::FeedLoaded(feed));
    }
}

/// The wire spelling of a degradation flag, the inverse of `Degraded::from_wire`.
///
/// The SDK has no `to_wire` (the client never sends these flags back), and
/// rebuilding a feed needs one so a rebuilt `FeedState` keeps the badges it showed.
fn degraded_wire(flag: Degraded) -> String {
    match flag {
        Degraded::ColdStart => "cold_start",
        Degraded::StaleSimilarity => "stale_similarity",
        Degraded::NoServerSync => "no_server_sync",
    }
    .to_owned()
}

/// Loads the newest feed page and clears the refresh flag.
pub(super) async fn load_first_page(sdk: &WebSdk, ui: FeedUi) {
    match sdk.load_feed(None).await {
        Ok(feed) => remember_feed(&feed),
        Err(error) => ui.error.set(Some(error.to_string())),
    }
    ui.refreshing.set(false);
}

/// Keeps the snapshot id that event batches must carry, and names the list cards join.
///
/// The impressions themselves are reported per card, when the card leaves the screen
/// (D36): only there is dwell known, and only once can the row be written without
/// doubling the denominator every rate is divided by.
fn remember_feed(feed: &AccountFeed) {
    FEED_ID.with_borrow_mut(|slot| *slot = Some(feed.feed_id.clone()));
    feed_ui().error.set(None);
    impressions::set_feed(&feed.feed_id);
}

/// RFC 3339 timestamp from the browser clock; the server clamps it to ±10 minutes.
fn client_timestamp() -> String {
    js_sys::Date::new_0().to_iso_string().into()
}
