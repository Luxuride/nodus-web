//! The card's "copy profile link" action and the toast it reports through.
//!
//! Apart from `item` because the clipboard is a browser capability, not card
//! layout: this is the only file that touches `navigator.clipboard`, and the only
//! place that knows what a profile URL looks like. `item` renders the [`ShareReport`]
//! it gets back; nothing here draws anything.

/// How long a clipboard toast stays on screen: long enough to read, short enough
/// that it never becomes something the user has to dismiss.
pub(super) const TOAST_MILLIS: u64 = 2_500;

/// What the clipboard button last reported, and whether it worked.
#[derive(Debug, Clone)]
pub(super) struct ShareReport {
    /// `true` for success: the two outcomes need different alert tones.
    pub(super) ok: bool,
    /// Plain text, either the success line or the browser's refusal.
    pub(super) message: String,
}

/// Copies the account's public profile link to the clipboard.
///
/// # Why the host comes from the session
///
/// `FeedAccount` carries no instance host, so the signed-in session's host is the
/// only one available. That is also the right guess in practice: the feed is built
/// from this instance's recommendations, so a profile is on this instance unless the
/// account is remote, and a remote `acct` (`user@other.example`) still resolves
/// through the local instance's `/@user@other.example` page.
///
/// The error is a user-facing sentence, because the caller only has the toast to
/// say anything with: a refused permission and a missing window look the same to it.
pub(super) async fn copy_profile_link(host: &str, acct: &str) -> Result<(), String> {
    let url = profile_url(host, acct).ok_or_else(|| "Not signed in.".to_owned())?;
    let clipboard = web_sys::window()
        .ok_or_else(|| "This browser has no clipboard access.".to_owned())?
        .navigator()
        .clipboard();
    wasm_bindgen_futures::JsFuture::from(clipboard.write_text(&url))
        .await
        .map_err(|_| "The browser refused clipboard access.".to_owned())?;
    Ok(())
}

/// `https://{host}/@{acct}`, or `None` when neither argument names a host.
///
/// `Session::host` is whatever the user typed at sign-in, so it may carry a scheme
/// or a trailing slash; both are stripped so the result is one well-formed URL. An
/// empty host falls back to the domain inside `acct`, which is what a pasted
/// `user@example.social` handle carries anyway.
fn profile_url(host: &str, acct: &str) -> Option<String> {
    let host = host.trim();
    let host = host.split_once("://").map_or(host, |(_, rest)| rest);
    let host = host.trim_end_matches('/');
    let host = if host.is_empty() {
        acct.rsplit_once('@').map(|(_, domain)| domain)?
    } else {
        host
    };
    Some(format!("https://{host}/@{acct}"))
}
