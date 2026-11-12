#![expect(dead_code, reason = "the shared thread does not filter yet")]

//! What the reader asked never to see, applied where the posts are cached.
//!
//! Mastodon filters on the server for the pages it assembles, but this client renders
//! finished posts out of SDK state: a thread put together from ancestors, a focused post
//! and replies, a timeline served from a page younger than its TTL. None of those passes
//! through the server's eye again, so the matching rule has to run here too — otherwise a
//! filter silently stops applying the moment the client starts working from cache.
//!
//! The rule itself lives in `nodus_client_core::model::filter` (`Filter::matches`), next to
//! the wire type; this module only decides *which* filters apply to *what*.

use leptos::prelude::*;
use nodus_client_core::model::{Filter, FilterContext, Status};
use nodus_client_core::model::{Verdict, judge as judge_text};
use nodus_client_sdk::page::{PageItems, PageKind};

use crate::routes::feed::posts::plain_text;
use crate::routes::status::StatusCard;
use crate::state::bridge::{pages, revision};

/// Asks the stored filters about one post.
///
/// Reads only: the filter list comes from the SDK's page store (`PageKind::Filters`),
/// which the settings screen keeps current, so a filter added a minute ago already
/// applies to a timeline the reader never reloads. The matching itself is
/// [`nodus_client_core::model::filter::judge`], beside the wire type, so a cache-filled
/// thread and a server-filled timeline cannot disagree.
#[must_use]
pub fn judge(status: &Status, context: FilterContext) -> Verdict {
    let filters: Vec<Filter> = pages::items(&PageKind::Filters, |items| match items {
        PageItems::Filters(filters) => Some(filters.clone()),
        _ => None,
    })
    .map(|(filters, _)| filters)
    .unwrap_or_default();

    let mut text = plain_text(&status.content);
    if !status.spoiler_text.is_empty() {
        text.push('\n');
        text.push_str(&status.spoiler_text);
    }
    judge_text(&filters, &text, context)
}

/// One post, rendered through the reader's filters.
///
/// A warned post is folded behind a disclosure that names the filter, so a reader who
/// expected it opens it in one click and a reader who did not is never shown it. A hidden
/// post paints nothing at all.
#[component]
pub fn Filtered(status: Status, context: FilterContext) -> impl IntoView {
    let revision = revision();
    let status = StoredValue::new(status);
    let verdict = move || {
        revision.get();
        judge(&status.get_value(), context)
    };

    view! {
        {move || match verdict() {
            Verdict::Hidden => ().into_any(),
            Verdict::Visible => view! { <StatusCard status=status.get_value()/> }.into_any(),
            Verdict::Warn(title) => {
                view! {
                    <details class="card border border-base-300 bg-base-200">
                        <summary class="cursor-pointer px-3 py-2 text-sm opacity-80">
                            "Hidden by your filter \u{201c}" {title} "\u{201d}. Show anyway?"
                        </summary>
                        <StatusCard status=status.get_value()/>
                    </details>
                }
                .into_any()
            }
        }}
    }
}
