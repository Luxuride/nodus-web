//! The accounts this browser can switch between.
//!
//! One person often holds several fediverse identities — a personal account on one
//! instance, a work account on another — and a client that can only remember one makes
//! them sign out to read the other timeline. The store keeps a small list, the active
//! key, and nothing else: the token belongs to the account, the state belongs to the
//! SDK instance built for it, and neither is shared.
//!
//! The format is a line per account rather than JSON, because it must survive a
//! corrupted entry: a line that cannot be parsed is dropped, and losing one account
//! record is a re-login, not a broken client. Values escape `%`, CR and LF, the three
//! characters the separator uses.

const LIST_KEY: &str = "nodus.accounts/1";
const ACTIVE_KEY: &str = "nodus.account";
const FORMAT: &str = "nodus-accounts/1";

/// One identity this browser holds a token for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    /// Instance the token was issued by, as typed by the user.
    pub host: String,
    /// Bearer token for that instance.
    pub token: String,
    /// The instance's id for the account.
    pub id: String,
    /// `user@host`, for display.
    pub acct: String,
}

impl Account {
    /// Stable key for this identity: the same account on two instances is two keys,
    /// because the tokens are not interchangeable.
    #[must_use]
    pub fn key(&self) -> String {
        key(&self.host, &self.id)
    }
}

/// Stable key for an identity.
#[must_use]
pub fn key(host: &str, id: &str) -> String {
    format!("{host}|{id}")
}

/// Every stored account, oldest first.
#[must_use]
pub fn load() -> Vec<Account> {
    let Some(raw) = storage().and_then(|storage| storage.get_item(LIST_KEY).ok().flatten()) else {
        return Vec::new();
    };
    let mut lines = raw.lines();
    if lines.next() != Some(FORMAT) {
        return Vec::new();
    }
    lines
        .filter(|line| !line.trim().is_empty())
        .filter_map(parse_line)
        .collect()
}

/// Replace the stored list.
///
/// An empty list removes the key instead of writing a header: a browser that has never
/// signed in should look like one, not like an empty database.
pub fn store(accounts: &[Account]) {
    let Some(storage) = storage() else {
        return;
    };
    if accounts.is_empty() {
        let _ = storage.remove_item(LIST_KEY);
        let _ = storage.remove_item(ACTIVE_KEY);
        return;
    }
    let mut out = String::from(FORMAT);
    out.push('\n');
    for account in accounts {
        out.push_str(&escape(&account.host));
        out.push('\t');
        out.push_str(&escape(&account.token));
        out.push('\t');
        out.push_str(&escape(&account.id));
        out.push('\t');
        out.push_str(&escape(&account.acct));
        out.push('\n');
    }
    let _ = storage.set_item(LIST_KEY, &out);
}

/// The key of the account the app is showing, if any.
#[must_use]
pub fn active() -> Option<String> {
    storage().and_then(|storage| storage.get_item(ACTIVE_KEY).ok().flatten())
}

/// Record which account is on screen.
pub fn set_active(key: Option<&str>) {
    let Some(storage) = storage() else {
        return;
    };
    match key {
        Some(key) => {
            let _ = storage.set_item(ACTIVE_KEY, key);
        }
        None => {
            let _ = storage.remove_item(ACTIVE_KEY);
        }
    }
}

/// Add an account, replacing an earlier record with the same key.
///
/// Replacing rather than appending is what a re-login does: the same identity with a new
/// token is one row, or the switcher would offer the same account twice.
pub fn insert(accounts: &mut Vec<Account>, account: Account) {
    match accounts.iter().position(|held| held.key() == account.key()) {
        Some(index) => accounts[index] = account,
        None => accounts.push(account),
    }
}

fn parse_line(line: &str) -> Option<Account> {
    let mut fields = line.split('\t');
    let host = unescape(fields.next()?)?;
    let token = unescape(fields.next()?)?;
    let id = unescape(fields.next()?)?;
    let acct = unescape(fields.next()?)?;
    if host.is_empty() || token.is_empty() || id.is_empty() {
        return None;
    }
    Some(Account {
        host,
        token,
        id,
        acct,
    })
}

fn escape(value: &str) -> String {
    value
        .replace('%', "%25")
        .replace('\n', "%0A")
        .replace('\r', "%0D")
}

fn unescape(value: &str) -> Option<String> {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(character) = chars.next() {
        if character != '%' {
            out.push(character);
            continue;
        }
        let high = chars.next()?;
        let low = chars.next()?;
        match (high, low) {
            ('2', '5') => out.push('%'),
            ('0', 'A') => out.push('\n'),
            ('0', 'D') => out.push('\r'),
            _ => return None,
        }
    }
    Some(out)
}

fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}
