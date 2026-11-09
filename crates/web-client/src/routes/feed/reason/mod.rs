//! Turning the server's machine-readable reason into text a person can act on.
//!
//! The feed's promise is "people you can explain": an item the user cannot explain
//! is an item they will not trust. This module is the only place that wording lives,
//! so the badge, its hover text and the sample-followers line stay consistent.

use std::fmt::Write as _;

use nodus_sdk::Degraded;
use nodus_shared::Reason;

/// Turns the server's machine code into one line of text.
///
/// The codes come from the recommender and can grow without a client release, so
/// an unknown code must never reach the user: the fallback says what every item in
/// the list has in common instead of printing `followed_by_similar_users`.
pub(super) fn reason_line(reason: &Reason) -> String {
    match reason.code.as_str() {
        "followed_by_similar_users" | "similar_user" => similar_line(reason.support),
        "multiple_signals" => "Matched by more than one signal".to_owned(),
        "affinity" => "Similar to accounts you engage with".to_owned(),
        "popular_fallback" | "popular" => "Popular on your instance".to_owned(),
        "author_you_should_follow" => "Author of posts you engage with".to_owned(),
        _ => "Recommended for you".to_owned(),
    }
}

/// The badge's `title`: the raw reason, for the user who wants to know more.
///
/// Plain text on purpose — it is rendered as an attribute, so nothing in it can
/// become markup — and it keeps the machine terms (`code`, `support`, `via`)
/// because they are what the user would quote when asking why an item appeared.
pub(super) fn reason_title(reason: &Reason) -> String {
    let mut title = format!("Why this: {} (code: {})", reason_line(reason), reason.code);
    if reason.support > 0 {
        let _ = write!(title, ", support: {}", reason.support);
    }
    if let Some(via) = reason.via.as_deref().filter(|via| !via.trim().is_empty()) {
        let _ = write!(title, ", via: {via}");
    }
    title
}

/// "followed by @a, @b": the social proof behind an item, when the server sent it.
///
/// Names are capped at three so the line never wraps into the note; the remainder
/// becomes "+N", which is the part a user can act on ("many people" vs "two").
/// Returns `None` rather than an empty string, so the caller can skip the element.
pub(super) fn sample_followers_line(reason: &Reason) -> Option<String> {
    let names: Vec<&str> = reason
        .sample_followers
        .iter()
        .map(String::as_str)
        .map(str::trim)
        .filter(|handle| !handle.is_empty())
        .collect();
    if names.is_empty() {
        return None;
    }

    let shown: Vec<String> = names
        .iter()
        .take(3)
        .map(|handle| {
            if handle.starts_with('@') {
                (*handle).to_owned()
            } else {
                format!("@{handle}")
            }
        })
        .collect();
    let hidden = names.len().saturating_sub(shown.len());

    let mut line = format!("followed by {}", shown.join(", "));
    if hidden > 0 {
        let _ = write!(line, " +{hidden}");
    }
    Some(line)
}

/// "Followed by N people with similar taste", with the count left out when it adds
/// nothing.
pub(super) fn similar_line(support: u8) -> String {
    match support {
        0 | 1 => "Followed by someone with similar taste".to_owned(),
        count => format!("Followed by {count} people with similar taste"),
    }
}

/// The follower count, singular or plural.
pub(super) fn follower_label(count: i64) -> String {
    match count {
        1 => "1 follower".to_owned(),
        _ => format!("{count} followers"),
    }
}

/// One line of text for the first active degradation flag.
///
/// Muted body text, not a banner: a weaker ranking is a caveat about the list, not
/// an error that stops the user.
pub(crate) fn degraded_line(reason: Degraded) -> &'static str {
    match reason {
        Degraded::ColdStart => "Too few signals yet: these are the instance's cold-start picks.",
        Degraded::StaleSimilarity => {
            "The similarity data is a few hours old, so the order may be off."
        }
        Degraded::NoServerSync => {
            "Background sync is off, so these are ranked from this session only."
        }
    }
}
