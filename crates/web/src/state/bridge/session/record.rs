//! The hand-written `localStorage` record for a session.
//!
//! Kept apart from the session verbs because the format changes for a different
//! reason than the verbs do: this file is the only place that parses or writes the
//! stored bytes, so the PKCE + `IndexedDB` replacement in
//! `web/docs/03-data-layer.md` can replace it without touching sign-in or restore.

/// `localStorage` key holding the milestone-1 session record.
const SESSION_KEY: &str = "nodus.session";

/// Marks the storage format, so a future change signs the user in again instead of
/// silently misparsing a record written by an older build.
const SESSION_FORMAT: &str = "nodus-session/1";

/// The stored record for a signed-in session.
///
/// It carries no account name: restoring asks the instance who the token belongs to
/// (`Sdk::sign_in`), so a remembered `acct` could only ever disagree with the truth.
#[derive(Debug, Clone)]
pub(super) struct StoredSession {
    /// Instance host, as the user typed it.
    pub(super) host: String,
    /// OAuth bearer token for the instance and for the recommendation API.
    pub(super) token: String,
}

/// Writes the record. A storage failure (full, disabled, private mode) is not
/// fatal: the session works for this page load and the next one asks again.
pub(super) fn store(host: &str, token: &str) {
    if let Some(storage) = local_storage() {
        let _ = storage.set_item(SESSION_KEY, &encode(host, token));
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
/// Three lines: the prefix, the host, the token, with `%`, `\n` and `\r` escaped as
/// `%25`, `%0A`, `%0D`. That is the whole format, and it is hand-written on purpose:
/// this is the only structured value the client stores, and it is not worth a
/// serialization framework for two strings.
fn encode(host: &str, token: &str) -> String {
    let mut out = String::from(SESSION_FORMAT);
    for field in [host, token] {
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
    // A record without a token is not a session, whatever else it says.
    if token.is_empty() {
        return None;
    }
    Some(StoredSession { host, token })
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
