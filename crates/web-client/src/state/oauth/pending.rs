//! The half-finished sign-in record, kept in `sessionStorage` across the redirect.
//!
//! `sessionStorage` rather than `localStorage`, and not the session record: the
//! record is only meaningful in the tab that started the flow, must not survive the
//! tab (a code left behind is a code left to replay), and is useless on any other
//! page. The access token that ends the flow still goes to `localStorage` through
//! `bridge::save_session`, exactly as before.

use core::fmt;

use web_sys::Storage;

/// `sessionStorage` key.
///
/// The key carries the format version, so a later record shape can use a new key and
/// this build simply never sees the old value — that is why there is no separate
/// format-marker line here, unlike the session record (`state/bridge/session/record.rs`).
const PENDING_KEY: &str = "nodus-oauth/3";

/// What the callback needs, and nothing else.
///
/// No token: at this point there is none. The verifier and `state` are secrets for
/// the life of the flow (the verifier is the PKCE proof, `state` is the CSRF check),
/// so neither is ever logged or displayed.
/// What the flow is for.
///
/// A sign-in replaces the session; an admin grant only adds a second token, so the
/// callback has to know which of the two it is finishing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Purpose {
    /// The ordinary sign-in.
    SignIn,
    /// Asking for `admin:read admin:write` on top of an existing session.
    Admin,
}

impl Purpose {
    fn as_str(self) -> &'static str {
        match self {
            Self::SignIn => "sign-in",
            Self::Admin => "admin",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value {
            "sign-in" => Some(Self::SignIn),
            "admin" => Some(Self::Admin),
            _ => None,
        }
    }
}

pub(crate) struct Pending {
    /// Instance host, already normalised by `oauth::begin`.
    pub(crate) host: String,
    /// The public client id this host issued for the app.
    pub(crate) client_id: String,
    /// The client secret the same registration returned; `/oauth/token` requires it.
    pub(crate) client_secret: String,
    /// PKCE verifier whose S256 challenge went to `/oauth/authorize`.
    pub(crate) verifier: String,
    /// The `state` sent to `/oauth/authorize`; the callback must echo it.
    pub(crate) state: String,
    /// Registered redirect URI, echoed to `/oauth/token` byte-for-byte.
    pub(crate) redirect_uri: String,
    /// What the callback should do with the token.
    pub(crate) purpose: Purpose,
}

impl fmt::Debug for Pending {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Same rule as the transport's `Token`: the secret fields are not printable,
        // because `{:?}` in a log line is the usual way one leaks.
        f.debug_struct("Pending")
            .field("host", &self.host)
            .field("redirect_uri", &self.redirect_uri)
            .finish_non_exhaustive()
    }
}

/// Stores `pending`, replacing any earlier record.
///
/// A storage failure is fatal here, unlike in [`crate::state::bridge::save_session`]:
/// without the record the callback cannot verify `state` or prove the verifier, so
/// continuing would end in a confusing error on the instance's redirect.
pub(crate) fn save(pending: &Pending) -> Result<(), String> {
    let storage = storage().ok_or_else(|| STORAGE_BLOCKED.to_owned())?;
    storage
        .set_item(PENDING_KEY, &encode(pending))
        .map_err(|_| STORAGE_BLOCKED.to_owned())
}

/// Reads the record and deletes it in the same step.
///
/// Single use is the point: once the callback has read it, a replayed code (a
/// reload, a shared URL) finds nothing to compare against and is refused.
pub(crate) fn take() -> Option<Pending> {
    let storage = storage()?;
    let raw = storage.get_item(PENDING_KEY).ok()??;
    let pending = decode(&raw);
    // Removed even when the record does not parse: it is ours or it is garbage, and
    // either way keeping it would let the next callback try it again.
    let _ = storage.remove_item(PENDING_KEY);
    pending
}

/// The blocked-storage message, phrased with the way out.
const STORAGE_BLOCKED: &str = "This browser blocks sessionStorage for this page, which the sign-in redirect needs. \
     Allow site data, or use the token fallback below.";

/// `sessionStorage`, when the browser allows it (it throws in some private modes).
fn storage() -> Option<Storage> {
    web_sys::window()?.session_storage().ok()?
}

/// Serializes the record: six fields and the purpose, one per line, escaped by
/// [`escape_into`].
fn encode(pending: &Pending) -> String {
    let purpose = pending.purpose.as_str().to_owned();
    let fields = [
        &pending.host,
        &pending.client_id,
        &pending.client_secret,
        &pending.verifier,
        &pending.state,
        &pending.redirect_uri,
        &purpose,
    ];
    let mut out = String::new();
    for (index, field) in fields.iter().enumerate() {
        if index > 0 {
            out.push('\n');
        }
        escape_into(&mut out, field.as_str());
    }
    out
}

/// Parses a record written by [`encode`], rejecting anything else.
///
/// A rejected record is not an error message: it means "no sign-in in progress", and
/// the callback says so in its own words.
fn decode(raw: &str) -> Option<Pending> {
    let mut lines = raw.split('\n');
    let pending = Pending {
        host: unescape(lines.next()?)?,
        client_id: unescape(lines.next()?)?,
        client_secret: unescape(lines.next()?)?,
        verifier: unescape(lines.next()?)?,
        state: unescape(lines.next()?)?,
        redirect_uri: unescape(lines.next()?)?,
        purpose: Purpose::parse(&unescape(lines.next()?)?)?,
    };
    // More lines than fields means this was written by something else.
    if lines.next().is_some() {
        return None;
    }
    // A record with an empty field cannot complete the exchange, so treat it as
    // absent rather than send an empty `code_verifier` to the instance.
    let fields = [
        &pending.host,
        &pending.client_id,
        &pending.client_secret,
        &pending.verifier,
        &pending.state,
        &pending.redirect_uri,
    ];
    if fields.iter().any(|field| field.is_empty()) {
        return None;
    }
    Some(pending)
}

/// Appends `field` with `%`, `\n` and `\r` escaped.
///
/// The same three escapes as the session record (`state/bridge/session/record.rs`),
/// copied rather than shared:
/// `bridge` is already past the size where another concern belongs in it, and both
/// formats are small enough to read in one screen.
fn escape_into(out: &mut String, field: &str) {
    for ch in field.chars() {
        match ch {
            '%' => out.push_str("%25"),
            '\n' => out.push_str("%0A"),
            '\r' => out.push_str("%0D"),
            other => out.push(other),
        }
    }
}

/// Reverses [`escape_into`]; `None` on a malformed escape.
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
