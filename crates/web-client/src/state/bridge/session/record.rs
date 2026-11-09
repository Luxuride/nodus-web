//! The hand-written `localStorage` record for a session.
//!
//! Kept apart from the session verbs because the format changes for a different
//! reason than the verbs do: this file is the only place that parses or writes the
//! stored bytes, so the PKCE + `IndexedDB` replacement in
//! `web/docs/03-data-layer.md` can replace it without touching sign-in or restore.

/// `localStorage` key holding the milestone-1 session record.
const SESSION_KEY: &str = "nodus.session";

/// The instance this browser last signed in to.
///
/// Separate from the session record and deliberately *not* cleared by sign-out: a post
/// link outlives a session, and `/status/{id}` carries no host, so without this the only
/// honest answer is "I cannot tell which instance holds that post".
const HOST_KEY: &str = "nodus.host";

/// Marks the storage format, so a future change signs the user in again instead of
/// silently misparsing a record written by an older build.
///
/// Version 2 added the account: the record now says *who* this browser is signed in
/// as, so the first paint can render the signed-in app instead of a sign-in screen
/// while the token is being checked. Version 1 records are ignored, which costs one
/// sign-in.
const SESSION_FORMAT: &str = "nodus-session/2";

/// The stored record for a signed-in session.
///
/// The account fields are what the instance reported when the session was created.
/// They are a claim, not the truth: the token is what the instance checks, so a
/// stale `acct` can only make the header cosmetically wrong, never grant access.
#[derive(Debug, Clone)]
pub(super) struct StoredSession {
    /// Instance host, as the user typed it.
    pub(super) host: String,
    /// OAuth bearer token for the instance and for the recommendation API.
    pub(super) token: String,
    /// Account id on that instance, as reported at sign-in.
    pub(super) account_id: String,
    /// `user@host`, for the header.
    pub(super) acct: String,
}

/// Writes the record. A storage failure (full, disabled, private mode) is not
/// fatal: the session works for this page load and the next one asks again.
pub(super) fn store(host: &str, token: &str, account_id: &str, acct: &str) {
    if let Some(storage) = local_storage() {
        let _ = storage.set_item(SESSION_KEY, &encode(host, token, account_id, acct));
    }
}

/// Removes the record, whatever it says.
pub(super) fn clear() {
    if let Some(storage) = local_storage() {
        let _ = storage.remove_item(SESSION_KEY);
    }
}

/// The stored record, if there is a well-formed one.
pub(super) fn load() -> Option<StoredSession> {
    let raw = local_storage()?.get_item(SESSION_KEY).ok()??;
    decode(&raw)
}

/// Serializes the record.
///
/// Four lines: the prefix, the host, the token, the account, with `%`, `\n` and `\r` escaped as
/// `%25`, `%0A`, `%0D`. That is the whole format, and it is hand-written on purpose:
/// this is the only structured value the client stores, and it is not worth a
/// serialization framework for two strings.
fn encode(host: &str, token: &str, account_id: &str, acct: &str) -> String {
    let mut out = String::from(SESSION_FORMAT);
    for field in [host, token, account_id, acct] {
        out.push('\n');
        for ch in field.chars() {
            match ch {
                '%' => out.push_str("%25"),
                '\n' => out.push_str("%0A"),
                '\r' => out.push_str("%0D"),
                other => out.push(other),
            }
        }
    }
    out
}

/// Parses a record written by [`encode`], rejecting anything else.
fn decode(raw: &str) -> Option<StoredSession> {
    let mut lines = raw.split('\n');
    if lines.next()? != SESSION_FORMAT {
        return None;
    }
    let host = unescape(lines.next()?)?;
    let token = unescape(lines.next()?)?;
    let account_id = unescape(lines.next()?)?;
    let acct = unescape(lines.next()?)?;
    // A record without a token is not a session, whatever else it says.
    if token.is_empty() {
        return None;
    }
    Some(StoredSession {
        host,
        token,
        account_id,
        acct,
    })
}

/// Reverses the escaping in [`encode`]; `None` on a malformed escape.
fn unescape(field: &str) -> Option<String> {
    let mut out = String::with_capacity(field.len());
    let mut chars = field.chars();
    while let Some(ch) = chars.next() {
        if ch != '%' {
            out.push(ch);
            continue;
        }
        let high = chars.next()?;
        let low = chars.next()?;
        let byte = u8::from_str_radix(&format!("{high}{low}"), 16).ok()?;
        out.push(match byte {
            b'%' => '%',
            b'\n' => '\n',
            b'\r' => '\r',
            // Only the three escapes this format defines round-trip; anything else
            // is a record we did not write.
            _ => return None,
        });
    }
    Some(out)
}

/// `localStorage`, when the browser allows it (it throws in some private modes).
fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

/// Remembers which instance this browser reads from.
pub(super) fn remember_host(host: &str) {
    if let Some(storage) = local_storage() {
        let _ = storage.set_item(HOST_KEY, host);
    }
}

/// The instance this browser last read from, if it ever did.
pub(super) fn remembered_host() -> Option<String> {
    local_storage()?
        .get_item(HOST_KEY)
        .ok()
        .flatten()
        .filter(|host| !host.trim().is_empty())
}
