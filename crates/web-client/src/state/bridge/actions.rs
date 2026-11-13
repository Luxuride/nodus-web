//! User actions against the instance: the verbs that are not state.
//!
//! The SDK deliberately owns *state* — session, cursors, the feed snapshot — and not
//! these. A favourite is a thing the user did, and what it returns is the instance's own
//! answer, so the view calls one of these, keeps the returned status, and re-renders from
//! it. Nothing here writes into `AppState`: an optimistic toggle that the server later
//! refused would be a lie the state machine cannot retract.

use core::future::Future;

use nodus_client_core::InstanceApi;
use nodus_client_core::actions::{
    AccountActions, Browse, FilterActions, HighlightActions, InstanceMeta, MediaActions,
    ScheduleActions, Search, StatusActions,
};
use nodus_client_core::model::{
    Account, Context, Conversation, Draft, Filter, FilterDraft, HistoryEntry, List, LocalId,
    MediaAttachment, PollDraft, ProfileUpdate, Relationship, SearchResults, Status,
};
use nodus_mastodon::BrowserInstance;

pub use nodus_client_core::model::Visibility;

use super::instance_transport;

/// The emoji this instance offers, asked for once per page load.
///
/// The set is large (hundreds of entries on a busy instance) and changes only when the
/// instance's cache does, so it is fetched on demand and kept for the life of the document
/// rather than on every keystroke that opens a suggestion list.
#[expect(
    clippy::future_not_send,
    reason = "the browser transport is single-threaded; the SDK is used from one task at a time"
)]
/// The languages this instance publishes in.
///
/// Cached for the document like the emoji catalogue: the list changes when an admin edits
/// the server's languages, not between two posts, and a composer that refetched it on every
/// open would spend a request per keystroke-sized decision.
pub async fn languages() -> Vec<nodus_client_core::model::Language> {
    thread_local! {
        static HELD: std::cell::RefCell<Option<Vec<nodus_client_core::model::Language>>> =
            const { std::cell::RefCell::new(None) };
    }
    if let Some(held) = HELD.with(|held| held.borrow().clone()) {
        return held;
    }
    let fresh = match instance_transport() {
        Some(transport) => call(transport.languages()).await,
        None => Err("the client is not installed".to_owned()),
    };
    match fresh {
        Ok(languages) => {
            HELD.with(|held| *held.borrow_mut() = Some(languages.clone()));
            languages
        }
        Err(message) => {
            web_sys::console::warn_1(&format!("nodus: languages unavailable: {message}").into());
            Vec::new()
        }
    }
}

/// The signed-in account's own posting language, when it set one.
///
/// This is the only truthful "default language" Mastodon exposes: an instance publishes the
/// languages it supports, not the one a writer posts in. `None` means "ask the instance",
/// which in practice means English or the instance's first published language.
pub async fn my_language() -> Option<String> {
    /// Three states, not two: "not asked yet" is different from "asked, and the account
    /// did not choose one" — collapsing them would re-ask on every composer.
    enum Held {
        Unknown,
        Answer(Option<String>),
    }

    thread_local! {
        static HELD: std::cell::RefCell<Held> = const { std::cell::RefCell::new(Held::Unknown) };
    }

    let cached = HELD.with(|held| match &*held.borrow() {
        Held::Unknown => None,
        Held::Answer(answer) => Some(answer.clone()),
    });
    if let Some(answer) = cached {
        return answer;
    }
    let fresh = match instance_transport() {
        Some(transport) => call(transport.credential_account())
            .await
            .ok()
            .and_then(|account| account.source.language),
        None => None,
    };
    HELD.with(|held| *held.borrow_mut() = Held::Answer(fresh.clone()));
    fresh
}

pub async fn custom_emojis() -> Vec<nodus_client_core::model::CustomEmoji> {
    thread_local! {
        static HELD: std::cell::RefCell<Option<Vec<nodus_client_core::model::CustomEmoji>>> =
            const { std::cell::RefCell::new(None) };
    }
    if let Some(held) = HELD.with(|held| held.borrow().clone()) {
        return held;
    }
    let fresh = match instance_transport() {
        Some(transport) => call(transport.custom_emojis()).await,
        None => Err("the client is not installed".to_owned()),
    };
    match fresh {
        Ok(emojis) => {
            HELD.with(|held| *held.borrow_mut() = Some(emojis.clone()));
            emojis
        }
        Err(message) => {
            web_sys::console::warn_1(
                &format!("nodus: could not read custom emojis: {message}").into(),
            );
            Vec::new()
        }
    }
}

/// The transport for the current session, or a message fit for inline display.
fn transport() -> Result<BrowserInstance, String> {
    instance_transport().ok_or_else(|| "Not signed in.".to_owned())
}

/// Runs one transport call, flattening its error into a display string.
async fn call<T>(
    future: impl Future<Output = Result<T, nodus_client_core::Error>>,
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
    /// A poll to attach; `None` for a plain post.
    pub poll: Option<PollDraft>,
    /// The language the post is written in; `None` lets the instance guess.
    pub language: Option<String>,
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
            poll: None,
            language: None,
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
            poll: self.poll.clone(),
            language: self.language.clone(),
        }
    }
}

/// Publishes a status.
pub async fn publish(status: &NewStatus) -> Result<Status, String> {
    call(transport()?.post_status(&status.draft())).await
}

/// What a thread publish managed to send, and why it stopped when it did.
pub struct ThreadOutcome {
    /// The posts the instance accepted, oldest first.
    pub statuses: Vec<Status>,
    /// The refusal that stopped the chain, if it stopped before the last post.
    pub error: Option<String>,
}

/// Publishes posts in order, each answering the one before.
///
/// A failure does not pretend the earlier posts were rolled back: the instance has them,
/// so the outcome carries them home and the composer can show the thread that exists.
pub async fn publish_thread(parts: &[NewStatus]) -> ThreadOutcome {
    let mut statuses = Vec::new();
    let mut parent = parts.first().and_then(|part| part.reply_to.clone());
    let transport = match transport() {
        Ok(transport) => transport,
        Err(message) => {
            return ThreadOutcome {
                statuses,
                error: Some(message),
            };
        }
    };
    for part in parts {
        let mut status = part.clone();
        status.reply_to = parent.clone();
        match call(transport.post_status(&status.draft())).await {
            Ok(posted) => {
                parent = Some(posted.id.clone());
                statuses.push(posted);
            }
            Err(message) => {
                let error = if statuses.is_empty() {
                    message
                } else {
                    format!(
                        "Posted {} of {} posts, then stopped: {message}",
                        statuses.len(),
                        parts.len()
                    )
                };
                return ThreadOutcome {
                    statuses,
                    error: Some(error),
                };
            }
        }
    }
    ThreadOutcome {
        statuses,
        error: None,
    }
}

/// Sets or replaces one attachment's alt text.
///
/// The instance answers with the media as it now stands, so the caller can replace its
/// copy rather than patch a field and hope the rest did not move.
pub async fn update_media(id: &str, description: &str) -> Result<MediaAttachment, String> {
    call(transport()?.update_media(id, description)).await
}

/// Rewrites a status the signed-in account wrote.
pub async fn edit(id: &str, status: &NewStatus) -> Result<Status, String> {
    call(transport()?.edit_status(id, &status.draft())).await
}

/// Uploads one attachment and returns what the instance made of it.
#[expect(
    dead_code,
    reason = "kept for the screen that will call it: the composer uploads through the editor path, the edit history panel and filter editing are still to come"
)]
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
#[expect(
    dead_code,
    reason = "kept for the screen that will call it: the composer uploads through the editor path, the edit history panel and filter editing are still to come"
)]
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

/// Saves the signed-in account's profile.
///
/// The answer is dispatched into the entity store rather than handed back to one screen:
/// the display name shows up on the profile, the composer and every card that mentions
/// the account, and only the store can reach all of them.
pub async fn update_profile(display_name: String, note: String) -> Result<Account, String> {
    let transport = transport()?;
    let update = ProfileUpdate::new().display_name(display_name).note(note);
    let account = call(transport.update_profile(&update)).await?;
    super::dispatch(nodus_client_sdk::Command::AccountsUpserted {
        accounts: vec![account.clone()],
    });
    Ok(account)
}

/// Creates a filter from a draft.
pub async fn create_filter(draft: &FilterDraft) -> Result<Filter, String> {
    call(transport()?.create_filter(draft)).await
}

/// Edits a filter.
///
/// Unused until the card grows an editor; the SDK keeps it because a filter whose
/// phrases cannot be corrected is a filter the reader must delete and retype.
#[expect(
    dead_code,
    reason = "kept for the screen that will call it: the composer uploads through the editor path, the edit history panel and filter editing are still to come"
)]
pub async fn update_filter(id: &str, draft: &FilterDraft) -> Result<Filter, String> {
    call(transport()?.update_filter(id, draft)).await
}

/// Deletes a filter, and with it its phrases.
pub async fn delete_filter(id: &str) -> Result<(), String> {
    call(transport()?.delete_filter(id)).await
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
pub async fn schedule(
    text: &str,
    visibility: nodus_client_core::model::Visibility,
    at: &str,
) -> Result<(), String> {
    let draft = Draft {
        text: text.to_owned(),
        visibility,
        in_reply_to_id: None,
        idempotency_key: Some(format!("nodus-{}", js_sys::Date::now())),
        media_ids: Vec::new(),
        spoiler_text: String::new(),
        sensitive: false,
        poll: None,
        language: None,
    };
    call(transport()?.schedule(&draft, at))
        .await
        .map(|_queued| ())
}

/// Cancels a queued post.
pub async fn cancel_scheduled(id: &str) -> Result<(), String> {
    call(transport()?.cancel_scheduled(id)).await
}

/// What the instance says about itself: name, version, contact, rules.
pub async fn instance_info() -> Result<nodus_client_core::model::InstanceInfo, String> {
    let transport = instance_transport().ok_or_else(|| "Not signed in.".to_owned())?;
    transport
        .instance_info()
        .await
        .map_err(|error| error.to_string())
}

/// Uploads one file and reports how much of it has left the browser.
///
/// `upload_media` in the SDK goes through `fetch`, which reports nothing until the
/// whole body is sent. This builds the same request (`media_upload_request`) and sends
/// it with `XMLHttpRequest`, whose `upload` progress events are the only real signal a
/// page can get.
pub async fn upload_media_with_progress(
    bytes: Vec<u8>,
    mime: &str,
    description: Option<&str>,
    on_progress: impl Fn(u8) + 'static,
) -> Result<MediaAttachment, String> {
    use wasm_bindgen::JsCast as _;
    use wasm_bindgen::prelude::Closure;

    let transport = instance_transport().ok_or("This session has no instance to upload to.")?;
    let request = transport
        .media_upload_request(&bytes, mime, description)
        .map_err(|error| error.to_string())?;
    let unsupported = || "This browser refused the upload.".to_owned();
    let xhr = web_sys::XmlHttpRequest::new().map_err(|_| unsupported())?;
    xhr.open_with_async(request.method.as_str(), request.url.as_str(), true)
        .map_err(|_| unsupported())?;
    for (name, value) in &request.headers {
        xhr.set_request_header(name, value)
            .map_err(|_| unsupported())?;
    }
    xhr.set_response_type(web_sys::XmlHttpRequestResponseType::Text);
    let upload = xhr.upload().map_err(|_| unsupported())?;
    let ticker =
        Closure::<dyn FnMut(web_sys::ProgressEvent)>::new(move |event: web_sys::ProgressEvent| {
            if event.length_computable() && event.total() > 0.0 {
                #[expect(
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss,
                    reason = "a share of the body is a whole percentage"
                )]
                let percent = ((event.loaded() / event.total()) * 100.0) as u8;
                on_progress(percent);
            }
        });
    upload.set_onprogress(Some(ticker.as_ref().unchecked_ref()));

    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        let loaded = {
            let resolve = resolve.clone();
            let xhr = xhr.clone();
            Closure::<dyn FnMut()>::new(move || {
                resolve.call1(&wasm_bindgen::JsValue::NULL, &xhr).ok();
            })
        };
        let failed = {
            let reject = reject.clone();
            Closure::<dyn FnMut()>::new(move || {
                reject.call0(&wasm_bindgen::JsValue::NULL).ok();
            })
        };
        xhr.set_onload(Some(loaded.as_ref().unchecked_ref()));
        xhr.set_onerror(Some(failed.as_ref().unchecked_ref()));
        loaded.forget();
        failed.forget();
    });
    let sent = xhr
        .send_with_opt_u8_array(request.body.as_deref())
        .map_err(|_| unsupported());
    sent?;
    ticker.forget();
    wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .map_err(|_| unsupported())?;

    let text = xhr
        .response_text()
        .map_err(|_| unsupported())?
        .unwrap_or_default();
    if xhr.status().map_err(|_| unsupported())? >= 400 {
        return Err(format!("The instance refused the file: {text}"));
    }
    serde_json::from_str::<MediaAttachment>(&text).map_err(|error| error.to_string())
}
