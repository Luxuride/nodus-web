//! Unposted writing, kept in the browser.
//!
//! A draft is not a status and not a scheduled one: nothing on the instance knows about it.
//! That is the whole point — it survives a reload or a closed tab without telling the server
//! anything the writer did not send. The store is therefore `localStorage`, one JSON array,
//! newest first, capped so a year of half-sentences cannot fill the quota.
//!
//! What is *not* stored: media and polls. Those are instance objects with ids and lifetimes;
//! a draft that restored a stale attachment id would fail to post for a reason the writer
//! cannot see. Text, visibility and language are enough to resume, and the rest is one
//! deliberate action away.

use nodus_instance::model::Visibility;

/// How many drafts are kept. Ten is what the popup can show without scrolling.
const CAP: usize = 10;

/// Where the list lives in `localStorage`.
const KEY: &str = "nodus-drafts";

/// One draft, as it was left.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, PartialEq)]
pub struct Saved {
    /// The body, exactly as typed.
    pub text: String,
    /// Visibility code, as the picker writes it (`"public"`, `"unlisted"`, …).
    pub visibility: String,
    /// Language code, or empty for "let the instance guess".
    pub language: String,
    /// Milliseconds since the epoch, for ordering and for the popup's age line.
    pub at: f64,
}

/// Every stored draft, newest first.
///
/// Unreadable storage yields an empty list rather than an error: a draft is a convenience,
/// and a corrupt entry must not cost the writer the composer.
pub fn all() -> Vec<Saved> {
    let Some(Ok(Some(raw))) = storage().map(|storage| storage.get_item(KEY)) else {
        return Vec::new();
    };
    serde_json::from_str(&raw).unwrap_or_default()
}

/// Keeps `text` as a draft if it is not blank.
///
/// Called when the composer goes away (a reload is a new document, so this is the close
/// path) and never from a publish: once the instance has the post, keeping a copy would
/// offer to send it twice.
pub fn keep(text: &str, visibility: Visibility, language: &str) {
    if text.trim().is_empty() {
        return;
    }
    write(
        &all()
            .into_iter()
            .filter(|draft| draft.text != text)
            .collect::<Vec<_>>(),
        text,
        visibility,
        language,
    );
}

/// Takes the draft at `index` out of the list and hands it back for the composer to fill.
pub fn take(index: usize) -> Option<Saved> {
    let mut drafts = all();
    if index >= drafts.len() {
        return None;
    }
    let chosen = drafts.remove(index);
    store(&drafts);
    Some(chosen)
}

/// Removes every draft.
pub fn clear() {
    store(&[]);
}

/// The visibility a stored code names, defaulting to public like the composer does.
#[must_use]
pub fn visibility(code: &str) -> Visibility {
    match code {
        "unlisted" => Visibility::Unlisted,
        "private" => Visibility::Private,
        "direct" => Visibility::Direct,
        _ => Visibility::Public,
    }
}

/// The stored code for a visibility.
#[must_use]
pub fn code(visibility: Visibility) -> String {
    visibility.as_str().to_owned()
}

/// A draft as one line, for the popup.
#[must_use]
pub fn summary(text: &str) -> String {
    let line = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if line.chars().count() > 60 {
        line.chars().take(59).chain(std::iter::once('…')).collect()
    } else {
        line
    }
}

fn write(existing: &[Saved], text: &str, visibility: Visibility, language: &str) {
    let mut drafts = Vec::with_capacity(existing.len() + 1);
    drafts.push(Saved {
        text: text.to_owned(),
        visibility: code(visibility),
        language: language.to_owned(),
        at: js_sys::Date::now(),
    });
    drafts.extend(existing.iter().cloned());
    drafts.truncate(CAP);
    store(&drafts);
}

fn store(drafts: &[Saved]) {
    let Some(storage) = storage() else { return };
    // A serialisation of plain strings cannot fail, and a full quota is not worth a panic
    // in the middle of someone's writing: both cases simply leave the previous list.
    if let Ok(json) = serde_json::to_string(drafts) {
        let _ = storage.set_item(KEY, &json);
    }
}

fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}
