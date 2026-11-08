//! One queued post, shared by the server paint and the client mount.
//!
//! The queue is read-only in both paints except for Cancel, so the row is the time, the
//! post's text, and what travels with it. The text is the SDK's `summary` — the first line,
//! as plain text — and it keeps its line breaks, because a post's own wrapping is part of
//! what the author wrote.
//!
//! The time arrives as a string: this crate reads no clock, so the server's stamp and the
//! client's relative age are the caller's business, and the row only has to leave the line
//! exactly as wide either of them needs. The attachment count comes from the ids the post
//! will carry, so a text-only post does not grow a "0" beside its time.
//!
//! Without `on_cancel` the button keeps its label and its box and is disabled, so the
//! server's row and the client's do not differ by a control's width.

use leptos::prelude::*;
use nodus_instance::model::ScheduledStatus;

/// The time line: when the post goes out, and how many files wait with it.
///
/// The separator belongs to the pair, not to either half: a caller with no time to show
/// (an unparsable date, on the server) still gets the attachment count without a leading
/// dot, and a post with no attachments keeps the time line exactly as the caller wrote it.
#[must_use]
fn when_line(age: &str, attachments: usize) -> String {
    let files = match attachments {
        0 => String::new(),
        1 => "1 attachment".to_owned(),
        count => format!("{count} attachments"),
    };
    match (age.is_empty(), files.is_empty()) {
        (false, false) => format!("{age} · {files}"),
        (false, true) => age.to_owned(),
        (true, false) => files,
        (true, true) => String::new(),
    }
}

/// One queued post: when it goes out, what it says, and the way to stop it.
///
/// `age` is the scheduled time in the caller's own words — the server's formatted stamp, or
/// the client's [`crate::relative_age`] — and `on_cancel` receives the scheduled status id.
#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "a Leptos component receives its props by value"
)]
pub fn ScheduledRow(
    /// The queued post this row shows.
    scheduled: ScheduledStatus,
    /// The scheduled time, formatted by the caller.
    age: String,
    /// Cancels the queued post.
    #[prop(optional)]
    on_cancel: Option<Callback<String>>,
) -> impl IntoView {
    let target = StoredValue::new(scheduled.id.clone());
    let when = when_line(&age, scheduled.params.media_ids.len());
    let preview = scheduled.summary();

    view! {
        <li class="flex items-start gap-3 p-3">
            <div class="min-w-0 flex-1">
                <p class="text-xs opacity-60">{when}</p>
                <p class="whitespace-pre-line text-sm">{preview}</p>
            </div>
            <button
                class="btn btn-ghost btn-xs"
                disabled=move || on_cancel.is_none()
                on:click=move |_| {
                    if let Some(on_cancel) = on_cancel {
                        on_cancel.run(target.get_value());
                    }
                }
            >
                "Cancel"
            </button>
        </li>
    }
}

#[cfg(test)]
mod tests {
    use super::when_line;

    #[test]
    fn a_text_only_post_keeps_the_callers_line() {
        assert_eq!(when_line("3h", 0), "3h");
    }

    #[test]
    fn attachments_are_counted_with_the_separator() {
        assert_eq!(when_line("3h", 1), "3h · 1 attachment");
        assert_eq!(when_line("3h", 4), "3h · 4 attachments");
    }

    #[test]
    fn a_missing_time_does_not_leave_a_leading_separator() {
        assert_eq!(when_line("", 2), "2 attachments");
        assert_eq!(when_line("", 0), "");
    }
}
