//! Small formatting helpers shared by the views.
//!
//! Shared rather than copied because a second copy is how "3h" and "3 h" end up
//! side by side in the same list.

/// A coarse, honest age: exact timestamps are one click away at the instance.
///
/// Parsing is deliberately shallow: the server sends RFC 3339, the browser clock is
/// the user's, and a wrong-by-seconds age is harmless while a panicking parse is not.
#[must_use]
pub fn relative_time(rfc3339: &str) -> String {
    nodus_view::relative_age(rfc3339, js_sys::Date::now())
}

/// The name to show for an account: the display name, or the handle when it is empty.
#[must_use]
pub fn display_name(display_name: &str, acct: &str) -> String {
    nodus_view::display_name(display_name, acct)
}
