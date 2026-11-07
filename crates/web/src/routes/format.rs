//! Small formatting helpers shared by the views.
//!
//! Shared rather than copied because a second copy is how "3h" and "3 h" end up
//! side by side in the same list.

/// A coarse, honest age: exact timestamps are one click away at the instance.
///
/// Parsing is deliberately shallow: the server sends RFC 3339, the browser clock is
/// the user's, and a wrong-by-seconds age is harmless while a panicking parse is not.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "minutes since a past instant, clamped non-negative, fit a u32 for centuries"
)]
#[must_use]
pub fn relative_time(rfc3339: &str) -> String {
    let stamp = js_sys::Date::parse(rfc3339);
    if !stamp.is_finite() {
        return String::new();
    }
    // A skewed client clock can put a just-published post in the future; "-3m" would
    // be a bug report, "now" would be the truth.
    let elapsed_minutes = (js_sys::Date::now() - stamp) / 60_000.0;
    let minutes = if elapsed_minutes <= 0.0 {
        0
    } else {
        elapsed_minutes as u32
    };
    match minutes {
        0 => "now".to_owned(),
        1..=59 => format!("{minutes}m"),
        60..=1439 => format!("{}h", minutes / 60),
        _ => format!("{}d", minutes / (60 * 24)),
    }
}

/// The name to show for an account: the display name, or the handle when it is empty.
#[must_use]
pub fn display_name(display_name: &str, acct: &str) -> String {
    if display_name.trim().is_empty() {
        acct.to_owned()
    } else {
        display_name.to_owned()
    }
}
