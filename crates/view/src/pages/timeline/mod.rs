//! A timeline: the list, its keys, its empty state and its "Load more".
//!
//! What is *not* here is the post. The client cards carry verbs, a filter verdict, a media
//! viewer and a clock; the server cards carry none of that. That disagreement is real, so it
//! is expressed once, as a [`CardRenderer`] the page calls, rather than twice as two lists.

use leptos::prelude::*;
use nodus_instance::model::Status;

use crate::components::card_renderer::CardRenderer;

/// A list of statuses, in the order the caller gave them.
///
/// The key is the status id: a keyed list with a duplicate key is a Leptos panic, not a
/// cosmetic bug, so callers must dedupe before handing the vector over.
#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "the renderer is cloned into a StoredValue, which copies the Arc, not moves the prop"
)]
pub fn StatusList(
    items: Signal<Vec<Status>>,
    card: CardRenderer,
    /// Whether a page is in flight for more, which disables the footer button.
    #[prop(into)]
    loading: Signal<bool>,
    /// Whether there is an older page to ask for.
    #[prop(into)]
    more: Signal<bool>,
    #[prop(optional)] on_more: Option<Callback<()>>,
    /// Said instead of the list when there is nothing to show and nothing coming.
    #[prop(optional, into)] empty: Option<String>,
) -> impl IntoView {
    // `StoredValue`: the renderer is called inside `<For>`, whose children closure must be
    // `Fn`; an `Arc` moved into it would be consumed by the first row.
    let card = StoredValue::new(std::sync::Arc::clone(&card));
    // `StoredValue`: the empty note is read inside a `fallback` closure, and a `String`
    // moved into it would make that closure `FnOnce`, which `Show` does not accept.
    let note = StoredValue::new(empty);

    view! {
        <Show
            when=move || !items.get().is_empty()
            fallback=move || {
                view! {
                    <p class="text-sm opacity-60">
                        {move || {
                            note.get_value().unwrap_or_else(|| "Nothing here yet.".to_owned())
                        }}
                    </p>
                }
            }
        >
            <ul class="flex flex-col gap-2">
                <For each=move || items.get() key=|status| status.id.clone() let:status>
                    {card.with_value(|card| card(status))}
                </For>
            </ul>
        </Show>
        <Show when=move || more.get()>
            <div class="flex justify-center py-2">
                <button
                    class="btn btn-sm"
                    disabled=move || loading.get()
                    on:click=move |_| {
                        if let Some(more) = on_more {
                            more.run(());
                        }
                    }
                >
                    {move || if loading.get() { "Loading…" } else { "Load more" }}
                </button>
            </div>
        </Show>
    }
}
