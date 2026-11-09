//! The admin grant: a second token, kept per instance, used only by moderation calls.
//!
//! A token without `admin:read` answers 403 to every `/api/v1/admin/*` request, so a
//! moderator signed in normally would see an empty Administration tab that looks like a
//! bug. The grant is separate on purpose: the ordinary session stays a token that cannot
//! moderate, and only the screens that need it reach for this one.
//!
//! A grant is per instance, not per account: switching accounts on the same instance
//! keeps it, because the instance is what issued it and the operator is the one who
//! approved the wider scope.

use core::cell::RefCell;

use web_sys::Storage;

/// `localStorage` key; the version lives in the key so a new record shape can simply
/// pick a new one and this build never sees the old value.
const KEY: &str = "nodus-admin/1";

/// One instance's grant.
struct Grant {
    host: String,
    token: String,
}

thread_local! {
    /// Parsed once, written back whole: the record is a handful of lines, and rewriting
    /// it is cheaper than reasoning about a partial update.
    static GRANTS: RefCell<Option<Vec<Grant>>> = const { RefCell::new(None) };
}

/// The admin token for `host`, when one was granted.
pub(super) fn load(host: &str) -> Option<String> {
    with_grants(|grants| {
        grants
            .iter()
            .find(|grant| grant.host == host)
            .map(|grant| grant.token.clone())
    })
}

/// Stores `token` for `host`, replacing an earlier grant for the same instance.
pub(super) fn store(host: &str, token: &str) {
    with_grants(|grants| {
        grants.retain(|grant| grant.host != host);
        grants.push(Grant {
            host: host.to_owned(),
            token: token.to_owned(),
        });
    });
    save();
}

/// Forgets the grant for `host`.
///
/// Called on sign-out of an instance: the token moderates, so it must not outlive the
/// session that the operator thinks they closed.
pub(super) fn clear(host: &str) {
    with_grants(|grants| grants.retain(|grant| grant.host != host));
    save();
}

fn with_grants<R>(f: impl FnOnce(&mut Vec<Grant>) -> R) -> R {
    GRANTS.with_borrow_mut(|slot| {
        if slot.is_none() {
            *slot = Some(read().unwrap_or_default());
        }
        f(slot.as_mut().expect("just filled"))
    })
}

fn storage() -> Option<Storage> {
    web_sys::window()?.local_storage().ok()?
}

/// Reads `<host>\t<token>` lines, dropping any it cannot parse.
fn read() -> Option<Vec<Grant>> {
    let raw = storage()?.get_item(KEY).ok()??;
    let grants = raw
        .lines()
        .filter_map(|line| {
            let (host, token) = line.split_once('\t')?;
            if host.is_empty() || token.is_empty() {
                return None;
            }
            Some(Grant {
                host: host.to_owned(),
                token: token.to_owned(),
            })
        })
        .collect();
    Some(grants)
}

fn save() {
    let Some(storage) = storage() else {
        return;
    };
    let text = with_grants(|grants| {
        grants
            .iter()
            .map(|grant| format!("{}\t{}", grant.host, grant.token))
            .collect::<Vec<_>>()
            .join("\n")
    });
    // A host and a token are both tab-free by construction (a domain has no whitespace,
    // a Mastodon token is base64url), so one `replace` is not needed here — unlike the
    // session record, whose fields can contain anything the operator typed.
    let _ = storage.set_item(KEY, &text);
}
