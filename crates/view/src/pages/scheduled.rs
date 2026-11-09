//! The scheduled screen: posts queued for later.
//!
//! This screen is the queue, not a way to add to it: a post is scheduled from the box that
//! also holds its text, so there is one place to write and one place to look at what is
//! waiting. The queue has no pages — it is read whole — so there is no paging control and no
//! cursor here, which is the one shape this screen does not share with the other lists.
//!
//! Everything the queue can do is cancel an entry, and that is the only verb it asks the
//! caller for. The row is shared with the server paint, which draws the same button with
//! nothing behind it.

use leptos::prelude::*;
use nodus_instance::model::ScheduledStatus;

use crate::chrome::Skeleton;
use crate::{PageHeader, Screen};
use crate::lists::scheduled_row::{ScheduledRow, ScheduledRowProps};

/// The queue of posts waiting to be published.
#[component]
pub fn ScheduledPage(
    /// The queued posts, in the order the instance returned them.
    items: Signal<Vec<ScheduledStatus>>,
    /// Whether a read is already on its way.
    loading: Signal<bool>,
    /// The last read's refusal, when it had one.
    error: Signal<Option<String>>,
    /// Formats a scheduled time the way the caller counts time. Absent when the caller has no
    /// clock to lend; the row then keeps the instance's own stamp.
    #[prop(optional)]
    age_of: Option<Callback<String, String>>,
    /// Cancels one queued post.
    #[prop(optional)]
    on_cancel: Option<Callback<String>>,
) -> impl IntoView {
    // The row's time line is the caller's, asked once per row so the page reads no clock. A
    // caller with no clock passes no `age_of`, and the queue still shows when each post goes
    // out: the stamp is the instance's, and the row keeps whatever it is given.
    let when_of = move |scheduled_at: &str| {
        age_of.map_or_else(
            || scheduled_at.to_owned(),
            |age_of| age_of.run(scheduled_at.to_owned()),
        )
    };

    view! {
        <Screen>
            <PageHeader
                title="Scheduled"
                description="Posts waiting to be published. Write one from the box above the home timeline."
            />

            {move || {
                error
                    .get()
                    .map(|message| {
                        view! {
                            <div class="alert alert-error py-2 text-sm"><span>{message}</span></div>
                        }
                    })
            }}

            // Only before the first answer: a held queue renders immediately, and an empty
            // queue is a fact, not a wait.
            <Show when=move || loading.get() && items.get().is_empty()>
                <Skeleton count=4/>
            </Show>

            <ul class="flex flex-col divide-y divide-base-300 rounded-box bg-base-100">
                <For each=move || items.get() key=|item| item.id.clone() let:item>
                    {
                        let age = when_of(&item.scheduled_at);
                        // The row is handed its whole set of props: view syntax can pass an
                        // affordance but not the absence of one, and the queue only knows what it
                        // was given.
                        ScheduledRow(ScheduledRowProps {
                            scheduled: item,
                            age,
                            on_cancel,
                        })
                    }
                </For>
            </ul>

            <Show when=move || !loading.get() && items.get().is_empty()>
                <p class="text-sm opacity-60">"Nothing is waiting."</p>
            </Show>
        </Screen>
    }
}
