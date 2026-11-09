//! The reader's filters: phrases that hide a post, or put it behind a warning.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_sdk::page::{PageItems, PageKind};

mod new;
mod row;

use self::new::NewFilterForm;
use self::row::FilterRow;

use crate::state::alive::Alive;
use crate::state::bridge::{pages, revision};

/// Settings card listing filters and offering to add one.
///
/// The server applies filters to timelines it returns; this screen is the editor, so the
/// list here is the account's own record rather than a local guess.
#[component]
pub fn FiltersCard() -> impl IntoView {
    let alive = Alive::new();
    let revision = revision();
    // Read out of the SDK's page store: this card is an editor, not a cache owner, so a
    // second visit paints the filters it already has instead of asking again.
    let filters = move || {
        revision.get();
        pages::items(&PageKind::Filters, |items| match items {
            PageItems::Filters(list) => Some(list),
            _ => None,
        })
        .map(|(list, _)| list)
        .unwrap_or_default()
    };
    let (busy, set_busy) = signal(true);
    let (error, set_error) = signal(None::<String>);

    // After a create or a delete the local copy is known to be wrong, which is the one
    // case that ignores the store's timestamp; on mount it may be trusted.
    let reload = Callback::new(move |()| {
        spawn_local(async move {
            let outcome = pages::refresh(PageKind::Filters).await;
            alive.if_alive(|| {
                if let Err(message) = outcome {
                    set_error.set(Some(message));
                }
                set_busy.set(false);
            });
        });
    });

    Effect::new(move |_| {
        spawn_local(async move {
            if let Err(message) = pages::ensure(PageKind::Filters).await {
                alive.if_alive(|| set_error.set(Some(message)));
            }
            alive.if_alive(|| set_busy.set(false));
        });
    });

    view! {
        <nodus_view::components::panel::Panel title="Filters">
                <p class="text-sm opacity-70">
                    "A filter hides a post, or shows it behind a warning. The instance applies \
                     it to timelines it returns."
                </p>

                <Show when=move || error.get().is_some()>
                    <p class="text-sm text-error">{move || error.get().unwrap_or_default()}</p>
                </Show>

                <ul class="flex flex-col gap-2">
                    <For each=filters key=|filter| filter.id.clone() let:filter>
                        <FilterRow filter=filter reload=reload/>
                    </For>
                </ul>

                <Show when=move || filters().is_empty() && !busy.get()>
                    <p class="text-sm opacity-70">"No filters yet."</p>
                </Show>

                <NewFilterForm on_created=Callback::new(move |()| reload.run(())) set_error=set_error/>
        </nodus_view::components::panel::Panel>
    }
}
