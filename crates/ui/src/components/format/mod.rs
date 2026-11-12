//! How old it is, who wrote it, what happened: the small pure answers both paints interpolate.
//!
//! Functions rather than components because the caller owns the clock and passes the value in.

/// Days from the civil date to the Unix epoch.
///
/// Howard Hinnant's `days_from_civil`, which is exact for every date the wire can carry and
/// needs no calendar crate: an age in hours does not justify a dependency.
pub fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let shifted_month = (month + 9) % 12;
    let day_of_year = (153 * shifted_month + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

/// Seconds since the Unix epoch for an RFC 3339 stamp, or `None` when it cannot be read.
///
/// Only the shape Mastodon sends is read (`YYYY-MM-DDTHH:MM:SS`, optionally with a fractional
/// part and a `Z`/offset tail, which is ignored); a stamp that does not match answers `None`
/// so the caller can render nothing rather than a wrong age.
#[must_use]
pub fn unix_seconds(rfc3339: &str) -> Option<i64> {
    let bytes = rfc3339.as_bytes();
    if bytes.len() < 19 {
        return None;
    }
    let number = |from: usize, to: usize| -> Option<i64> { rfc3339.get(from..to)?.parse().ok() };
    let year = number(0, 4)?;
    let month = number(5, 7)?;
    let day = number(8, 10)?;
    let hour = number(11, 13)?;
    let minute = number(14, 16)?;
    let second = number(17, 19)?;
    Some(days_from_civil(year, month, day) * 86_400 + hour * 3_600 + minute * 60 + second)
}

/// A coarse, honest age: exact timestamps are one click away at the instance.
///
/// Parsing is deliberately shallow and the answer is deliberately imprecise: the server and
/// the browser each compute it from the same instant, and a difference of seconds is fine
/// while a difference of *shape* ("3h" against "3 hours ago") reads as a page that changed
/// its mind. So both paints call this one function.
#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "minutes since a past instant, clamped non-negative, fit a u32 for centuries"
)]
pub fn relative_age(created_at: &str, now_millis: f64) -> String {
    let Some(stamp) = unix_seconds(created_at) else {
        return String::new();
    };
    #[expect(
        clippy::cast_precision_loss,
        reason = "seconds since 1970 in a timestamp are exact in f64 well past this century"
    )]
    let published = stamp as f64;
    let elapsed_minutes = (now_millis / 60_000.0) - published / 60.0;
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

/// What kind of attention a notification was.
#[must_use]
pub fn describe_notification(kind: nodus_client_core::model::NotificationKind) -> &'static str {
    use nodus_client_core::model::NotificationKind;
    match kind {
        NotificationKind::Follow => "followed you",
        NotificationKind::FollowRequest => "asked to follow you",
        NotificationKind::Reblog => "boosted your post",
        NotificationKind::Favourite => "favourited your post",
        NotificationKind::Mention => "mentioned you",
        NotificationKind::Poll => "A poll you voted in has ended",
        NotificationKind::Status => "posted",
        NotificationKind::AdminSignUp => "A new account signed up",
        NotificationKind::AdminReport => "A report needs review",
        NotificationKind::Unknown => "Something happened",
    }
}
