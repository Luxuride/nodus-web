//! Turning the server's machine-readable reason into text a person can act on.
//!
//! This is the only place the wording lives, so the badge, its hover text and the sample
//! followers line stay consistent between the painted card and the live one. The one line
//! that is *not* here is the degraded notice: it is keyed on a value from the SDK's state,
//! which this crate does not depend on, so the client maps that flag itself.
//!
//! Nothing here reads a transport, a clock or a store: the input is the reason the server
//! sent, and the output is a string.

use std::fmt::Write as _;

use nodus_shared::Reason;

/// Turns the server's machine code into one line of text.
///
/// The codes come from the recommender and can grow without a client release, so an unknown
/// code must never reach the user: the fallback says what every item in the list has in
/// common instead of printing `followed_by_similar_users`.
pub fn reason_line(reason: &Reason) -> String {
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
/// Plain text on purpose — it is rendered as an attribute, so nothing in it can become
/// markup — and it keeps the machine terms (`code`, `support`, `via`) because they are what
/// the user would quote when asking why an item appeared.
pub fn reason_title(reason: &Reason) -> String {
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
/// Names are capped at three so the line never wraps into the note; the remainder becomes
/// "+N", which is the part a user can act on ("many people" vs "two"). Returns `None` rather
/// than an empty string, so the caller can skip the element.
pub fn sample_followers_line(reason: &Reason) -> Option<String> {
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

/// "Followed by N people with similar taste", with the count left out when it adds nothing.
pub fn similar_line(support: u8) -> String {
    match support {
        0 | 1 => "Followed by someone with similar taste".to_owned(),
        count => format!("Followed by {count} people with similar taste"),
    }
}

/// The follower count, singular or plural.
pub fn follower_label(count: i64) -> String {
    match count {
        1 => "1 follower".to_owned(),
        _ => format!("{count} followers"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reason(code: &str) -> Reason {
        Reason {
            code: code.to_owned(),
            support: 0,
            sample_followers: Vec::new(),
            via: None,
        }
    }

    fn with_samples(code: &str, support: u8, samples: &[&str]) -> Reason {
        Reason {
            code: code.to_owned(),
            support,
            sample_followers: samples.iter().map(|name| (*name).to_owned()).collect(),
            via: None,
        }
    }

    #[test]
    fn known_codes_get_their_own_line() {
        assert_eq!(
            reason_line(&reason("multiple_signals")),
            "Matched by more than one signal"
        );
        assert_eq!(
            reason_line(&reason("affinity")),
            "Similar to accounts you engage with"
        );
        assert_eq!(
            reason_line(&reason("popular_fallback")),
            "Popular on your instance"
        );
        assert_eq!(
            reason_line(&reason("author_you_should_follow")),
            "Author of posts you engage with"
        );
    }

    #[test]
    fn unknown_codes_fall_back_to_a_sentence() {
        assert_eq!(
            reason_line(&reason("some_future_code")),
            "Recommended for you"
        );
    }

    #[test]
    fn similar_codes_count_the_support() {
        assert_eq!(similar_line(0), "Followed by someone with similar taste");
        assert_eq!(similar_line(1), "Followed by someone with similar taste");
        assert_eq!(similar_line(4), "Followed by 4 people with similar taste");
        assert_eq!(
            reason_line(&with_samples("similar_user", 3, &[])),
            similar_line(3)
        );
    }

    #[test]
    fn title_names_the_code_and_its_context() {
        assert_eq!(
            reason_title(&reason("affinity")),
            "Why this: Similar to accounts you engage with (code: affinity)"
        );
        let mut titled = with_samples("similar_user", 2, &[]);
        titled.via = Some("favourites".to_owned());
        assert_eq!(
            reason_title(&titled),
            "Why this: Followed by 2 people with similar taste (code: similar_user), support: 2, via: favourites"
        );
    }

    #[test]
    fn title_skips_empty_via() {
        let mut titled = reason("affinity");
        titled.via = Some("  ".to_owned());
        assert_eq!(
            reason_title(&titled),
            "Why this: Similar to accounts you engage with (code: affinity)"
        );
    }

    #[test]
    fn samples_are_capped_and_prefixed() {
        assert_eq!(
            sample_followers_line(&with_samples(
                "similar_user",
                5,
                &["bob", "@carol", "dave", "erin"]
            )),
            Some("followed by @bob, @carol, @dave +1".to_owned())
        );
    }

    #[test]
    fn empty_samples_are_no_line() {
        assert_eq!(
            sample_followers_line(&with_samples("similar_user", 1, &["", "  "])),
            None
        );
        assert_eq!(sample_followers_line(&reason("affinity")), None);
    }

    #[test]
    fn follower_label_is_singular_at_one() {
        assert_eq!(follower_label(0), "0 followers");
        assert_eq!(follower_label(1), "1 follower");
        assert_eq!(follower_label(12), "12 followers");
    }
}
