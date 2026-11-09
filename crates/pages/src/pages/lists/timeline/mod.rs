//! The selected list's timeline: its page of posts, the cursor that pages it, and the
//! refusals of both.
//!
//! The posts themselves are the caller's, because a timeline is read and paged by whoever has
//! a transport; what the panel owns is the shape — the same list, the same empty sentence and
//! the same load-more, so a painted page and a mounted one differ by nothing here.
//!
//! Only the client's card knows the age of a post, the addresses it opens and the verbs it
//! offers, so it arrives as a call for one status. Without one the panel draws the shared
//! read-only card, which is the same boxes with the verbs it cannot own.

use leptos::prelude::*;
use nodus_instance::model::Status;

use crate::status_card::{StatusCard, StatusCardProps};

/// The posts of the selected list.
#[component]
pub fn ListTimeline(
    /// The selected list; the panel is closed while it is `None`.
    list_id: Signal<Option<String>>,
    /// The posts that are already read.
    statuses: Signal<Vec<Status>>,
    /// The cursor for the next page, while there is one.
    cursor: Signal<Option<String>>,
    /// Whether a page is on its way.
    loading: Signal<bool>,
    /// The last refusal, shown under the posts.
    error: Signal<Option<String>>,
    /// Reads the next page.
    #[prop(optional)]
    on_more: Option<Callback<()>>,
    /// Formats a timestamp as an age, for the card this panel draws itself. Absent when the
    /// caller has no clock to lend, and the card then carries no age rather than a made up
    /// one.
    #[prop(optional)]
    age_of: Option<Callback<String, String>>,
    /// The client's own status card, drawn in place of the shared read-only one.
    card: Option<Callback<Status, AnyView>>,
) -> impl IntoView {
    view! {
        <Show when=move || list_id.get().is_some()>
            <ul class="flex flex-col gap-3">
                <For each=move || statuses.get() key=|status| status.id.clone() let:status>
                    {
                        card.map_or_else(
                            || {
                                let age = age_of.map_or_else(String::new, |age_of| {
                                    age_of.run(status.created_at.clone())
                                });
                                read_only_card(status.clone(), age)
                            },
                            |card| card.run(status.clone()),
                        )
                    }
                </For>
            </ul>

            <Show when=move || !loading.get() && statuses.get().is_empty() && error.get().is_none()>
                <p class="text-sm opacity-60">"Nobody on this list has posted yet."</p>
            </Show>

            <Show when=move || cursor.get().is_some()>
                <button
                    class="btn btn-ghost btn-sm w-full"
                    disabled=move || loading.get() || on_more.is_none()
                    on:click=move |_| {
                        if let Some(on_more) = on_more {
                            on_more.run(());
                        }
                    }
                >
                    "Load more"
                </button>
            </Show>

            <Show when=move || error.get().is_some()>
                <p class="text-error text-xs">{move || error.get().unwrap_or_default()}</p>
            </Show>
        </Show>
    }
}

/// The shared card for one post, with the seams this panel has no way to fill.
///
/// The props are built as a struct rather than through `view!` because every seam is
/// optional: a setter for one takes the bare value, and a panel that only has the option to
/// forward it would have to invent one.
fn read_only_card(status: Status, age: String) -> AnyView {
    let author_path = format!("/accounts/{}", status.account.id);
    let thread_path = format!("/status/{}", status.id);

    StatusCard(StatusCardProps {
        status,
        age,
        author_path,
        thread_path,
        on_navigate: None,
        on_open: None,
        on_boost: None,
        on_favourite: None,
        on_bookmark: None,
        on_reply: None,
        before_verbs: None,
        after_verbs: None,
        on_open_media: None,
        on_vote: None,
        interactive: false,
        body: None,
        extra: None,
    })
    .into_any()
}
