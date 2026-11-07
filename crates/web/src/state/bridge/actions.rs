//! User actions against the instance: the verbs that are not state.
//!
//! The SDK deliberately owns *state* — session, cursors, the feed snapshot — and not
//! these. A favourite is a thing the user did, and what it returns is the instance's own
//! answer, so the view calls one of these, keeps the returned status, and re-renders from
//! it. Nothing here writes into `AppState`: an optimistic toggle that the server later
//! refused would be a lie the state machine cannot retract.

use core::future::Future;

use nodus_instance::BrowserInstance;
use nodus_instance::InstanceApi;
use nodus_instance::actions::{
    AccountActions, Browse, FilterActions, HighlightActions, InstanceMeta, MediaActions,
    ScheduleActions,
    Search, StatusActions,
};
use nodus_instance::model::{
    Account, Context, Conversation, Draft, Filter, FilterDraft, HistoryEntry, List, LocalId, MediaAttachment,
    Page, PageParams, ProfileUpdate, Relationship, SearchResults, Status,
};

pub use nodus_instance::model::Visibility;

use super::instance_transport;

/// The transport for the current session, or a message fit for inline display.
fn transport() -> Result<BrowserInstance, String> {
    instance_transport().ok_or_else(|| "Not signed in.".to_owned())
}

/// Runs one transport call, flattening its error into a display string.
async fn call<T>(
    future: impl Future<Output = Result<T, nodus_instance::Error>>,
) -> Result<T, String> {
    future.await.map_err(|error| error.to_string())
}

/// The page size every list view asks for.
///
/// One constant, because a mismatch between "what was loaded" and "what the cursor
/// assumed" is how a timeline skips or repeats an entry.
pub const PAGE: u16 = 20;

/// One status.
pub async fn status(id: &str) -> Result<Status, String> {
    call(transport()?.status(id)).await
}

/// The ancestors and descendants of a status.
pub async fn thread(id: &str) -> Result<Context, String> {
    call(transport()?.status_context(id)).await
}

/// One account.
pub async fn account(id: &str) -> Result<Account, String> {
    let id = LocalId(id.to_owned());
    call(transport()?.account(&id)).await
}

/// The relationship between the signed-in account and `id`.
pub async fn relationship(id: &str) -> Result<Option<Relationship>, String> {
    let ids = [LocalId(id.to_owned())];
    let found = call(transport()?.relationships(&ids)).await?;
    Ok(found.into_iter().next())
}


/// Favourites or un-favourites a status.
pub async fn favourite(id: &str, on: bool) -> Result<Status, String> {
    let transport = transport()?;
    if on {
        call(transport.favourite(id)).await
    } else {
        call(transport.unfavourite(id)).await
    }
}

/// Boosts or un-boosts a status.
pub async fn reblog(id: &str, on: bool) -> Result<Status, String> {
    let transport = transport()?;
    if on {
        call(transport.reblog(id)).await
    } else {
        call(transport.unreblog(id)).await
    }
}

/// Bookmarks or un-bookmarks a status.
pub async fn bookmark(id: &str, on: bool) -> Result<Status, String> {
    let transport = transport()?;
    if on {
        call(transport.bookmark(id)).await
    } else {
        call(transport.unbookmark(id)).await
    }
}

/// Deletes a status the signed-in account wrote.
pub async fn delete_status(id: &str) -> Result<(), String> {
    call(transport()?.delete_status(id)).await
}

/// Everything a compose box can say beyond the text.
#[derive(Debug, Clone, PartialEq)]
pub struct NewStatus {
    /// The body, as typed.
    pub text: String,
    /// Requested visibility; the instance may still narrow it.
    pub visibility: Visibility,
    /// Parent status when replying.
    pub reply_to: Option<String>,
    /// Attachments already uploaded, in display order.
    pub media_ids: Vec<String>,
    /// Content warning; empty means none.
    pub spoiler_text: String,
    /// Whether the attachment sits behind the warning.
    pub sensitive: bool,
}

impl NewStatus {
    /// A reply or a post with nothing but text.
    pub fn text(text: impl Into<String>, visibility: Visibility) -> Self {
        Self {
            text: text.into(),
            visibility,
            reply_to: None,
            media_ids: Vec::new(),
            spoiler_text: String::new(),
            sensitive: false,
        }
    }

    /// The same, as a reply to `parent`.
    pub fn reply(text: impl Into<String>, parent: &str) -> Self {
        Self {
            reply_to: Some(parent.to_owned()),
            ..Self::text(text, Visibility::Public)
        }
    }

    /// The wire draft, with a fresh idempotency key per attempt.
    fn draft(&self) -> Draft {
        Draft {
            text: self.text.clone(),
            visibility: self.visibility,
            in_reply_to_id: self.reply_to.clone(),
            // A new key per attempt: the point is that a retried *request* cannot post
            // twice, not that the same text may never be posted twice.
            idempotency_key: Some(format!("nodus-{}", js_sys::Date::now())),
            media_ids: self.media_ids.clone(),
            spoiler_text: self.spoiler_text.clone(),
            sensitive: self.sensitive,
        }
    }
}

/// Publishes a status.
pub async fn publish(status: &NewStatus) -> Result<Status, String> {
    call(transport()?.post_status(&status.draft())).await
}

/// Rewrites a status the signed-in account wrote.
pub async fn edit(id: &str, status: &NewStatus) -> Result<Status, String> {
    call(transport()?.edit_status(id, &status.draft())).await
}

/// Uploads one attachment and returns what the instance made of it.
pub async fn upload_media(
    bytes: Vec<u8>,
    mime: String,
    description: Option<String>,
) -> Result<MediaAttachment, String> {
    call(transport()?.upload_media(&bytes, &mime, description.as_deref())).await
}

/// The account id the session belongs to, for "may I edit this" questions.
pub fn viewer_id() -> Option<String> {
    super::with_sdk(|sdk| sdk.state().session.get().map(|session| session.account_id))
}

/// Pending follow requests, newest first.
pub async fn follow_requests() -> Result<Vec<Account>, String> {
    call(transport()?.follow_requests()).await
}

/// Accepts a follow request.
pub async fn authorize_follow_request(id: &str) -> Result<Relationship, String> {
    call(transport()?.authorize_follow_request(&LocalId(id.to_owned()))).await
}

/// Refuses a follow request.
pub async fn reject_follow_request(id: &str) -> Result<(), String> {
    call(transport()?.reject_follow_request(&LocalId(id.to_owned()))).await
}





/// Votes on a poll and returns the status it belongs to.
pub async fn vote(poll_id: &str, choices: &[usize]) -> Result<Status, String> {
    call(transport()?.vote(poll_id, choices)).await
}

/// The edit history of a status, oldest first.
#[expect(dead_code, reason = "the edit-history panel lands with editing")]
pub async fn history(id: &str) -> Result<Vec<HistoryEntry>, String> {
    call(transport()?.history(id)).await
}

/// Follows or unfollows an account.
pub async fn follow(id: &str, on: bool) -> Result<Relationship, String> {
    let id = LocalId(id.to_owned());
    let transport = transport()?;
    // Follow lives on `InstanceApi` because the recommender's sync worker uses it, so
    // it needs no action trait; the branch is here because the two futures differ.
    if on {
        call(transport.follow(&id)).await
    } else {
        call(transport.unfollow(&id)).await
    }
}

/// Blocks or unblocks an account.
pub async fn block(id: &str, on: bool) -> Result<Relationship, String> {
    let id = LocalId(id.to_owned());
    let transport = transport()?;
    if on {
        call(transport.block(&id)).await
    } else {
        call(transport.unblock(&id)).await
    }
}

/// Mutes or unmutes an account.
pub async fn mute(id: &str, on: bool) -> Result<Relationship, String> {
    let id = LocalId(id.to_owned());
    let transport = transport()?;
    if on {
        call(transport.mute(&id)).await
    } else {
        call(transport.unmute(&id)).await
    }
}

/// Searches accounts, statuses and hashtags; `resolve` lets the instance fetch remote
/// matches it has never seen.
pub async fn search(query: &str, resolve: bool) -> Result<SearchResults, String> {
    call(transport()?.search(query, resolve)).await
}



/// The account's lists.
pub async fn lists() -> Result<Vec<List>, String> {
    call(transport()?.lists()).await
}

/// Creates a list and returns it.
pub async fn create_list(title: &str) -> Result<List, String> {
    call(transport()?.create_list(title)).await
}

/// Renames a list.
pub async fn rename_list(id: &str, title: &str) -> Result<List, String> {
    call(transport()?.rename_list(id, title)).await
}

/// Deletes a list; the people on it stay followed.
pub async fn delete_list(id: &str) -> Result<(), String> {
    call(transport()?.delete_list(id)).await
}

/// The accounts on a list.
pub async fn list_accounts(id: &str) -> Result<Vec<Account>, String> {
    call(transport()?.list_accounts(id)).await
}

/// Adds to or removes from a list, which differ only in the verb.
pub async fn set_list_member(list_id: &str, account_id: &str, member: bool) -> Result<(), String> {
    let transport = transport()?;
    let accounts = [LocalId(account_id.to_owned())];
    if member {
        call(transport.add_to_list(list_id, &accounts)).await
    } else {
        call(transport.remove_from_list(list_id, &accounts)).await
    }
}

/// One page of a list timeline.
pub async fn list_timeline(list_id: &str, after: Option<&str>) -> Result<Page<Status>, String> {
    let params = match after {
        Some(cursor) => PageParams::after(cursor, PAGE),
        None => PageParams::first(PAGE),
    };
    call(transport()?.list_timeline(list_id, params)).await
}

/// Saves the signed-in account's profile.
///
/// The answer is dispatched into the entity store rather than handed back to one screen:
/// the display name shows up on the profile, the composer and every card that mentions
/// the account, and only the store can reach all of them.
pub async fn update_profile(display_name: String, note: String) -> Result<Account, String> {
    let transport = transport()?;
    let update = ProfileUpdate::new().display_name(display_name).note(note);
    let account = call(transport.update_profile(&update)).await?;
    super::dispatch(nodus_sdk::Command::AccountsUpserted {
        accounts: vec![account.clone()],
    });
    Ok(account)
}

/// Every filter this account keeps.
pub async fn filters() -> Result<Vec<Filter>, String> {
    call(transport()?.filters()).await
}

/// Creates a filter from a draft.
pub async fn create_filter(draft: &FilterDraft) -> Result<Filter, String> {
    call(transport()?.create_filter(draft)).await
}

/// Edits a filter.
///
/// Unused until the card grows an editor; the SDK keeps it because a filter whose
/// phrases cannot be corrected is a filter the reader must delete and retype.
#[expect(dead_code, reason = "the settings card can add and delete, not yet edit")]
pub async fn update_filter(id: &str, draft: &FilterDraft) -> Result<Filter, String> {
    call(transport()?.update_filter(id, draft)).await
}

/// Deletes a filter, and with it its phrases.
pub async fn delete_filter(id: &str) -> Result<(), String> {
    call(transport()?.delete_filter(id)).await
}

/// The tags an account has featured on its profile.
pub async fn featured_tags(id: &str) -> Result<Vec<nodus_instance::model::FeaturedTag>, String> {
    call(transport()?.featured_tags(&nodus_instance::LocalId(id.to_owned()))).await
}

/// Features a tag on the signed-in account's profile.
pub async fn feature_tag(name: &str) -> Result<(), String> {
    call(transport()?.feature_tag(name)).await.map(|_tag| ())
}

/// Stops featuring a tag.
pub async fn unfeature_tag(id: &str) -> Result<(), String> {
    call(transport()?.unfeature_tag(id)).await
}


/// Marks a conversation as read.
pub async fn mark_conversation_read(id: &str) -> Result<(), String> {
    call(transport()?.mark_conversation_read(id)).await
}

/// Mutes or unmutes a conversation.
pub async fn mute_conversation(id: &str, muted: bool) -> Result<Conversation, String> {
    call(transport()?.mute_conversation(id, muted)).await
}


/// Queues a draft for `at`, in RFC 3339.
pub async fn schedule(text: &str, visibility: nodus_instance::model::Visibility, at: &str) -> Result<(), String> {
    let draft = Draft {
        text: text.to_owned(),
        visibility,
        in_reply_to_id: None,
        idempotency_key: Some(format!("nodus-{}", js_sys::Date::now())),
        media_ids: Vec::new(),
        spoiler_text: String::new(),
        sensitive: false,
    };
    call(transport()?.schedule(&draft, at)).await.map(|_queued| ())
}

/// Cancels a queued post.
pub async fn cancel_scheduled(id: &str) -> Result<(), String> {
    call(transport()?.cancel_scheduled(id)).await
}

/// Accounts the instance suggests, for finding people to follow.
pub async fn directory(local_only: bool, offset: u32) -> Result<Vec<Account>, String> {
    call(transport()?.directory(local_only, offset)).await
}

/// What the instance says about itself: name, version, contact, rules.
pub async fn instance_info() -> Result<nodus_instance::model::InstanceInfo, String> {
    let transport = instance_transport().ok_or_else(|| "Not signed in.".to_owned())?;
    transport
        .instance_info()
        .await
        .map_err(|error| error.to_string())
}
