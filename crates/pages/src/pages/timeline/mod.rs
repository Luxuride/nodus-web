//! A timeline: the list, its keys, its empty state and its "Load more".
//!
//! What is *not* here is the post. The client cards carry verbs, a filter verdict, a media
//! viewer and a clock; the server cards carry none of that. That disagreement is real, so it
//! is expressed once, as a [`CardRenderer`] the page calls, rather than twice as two lists.

use leptos::prelude::*;
use nodus_instance::model::Status;

use crate::components::card_renderer::CardRenderer;

pub mod banner;

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
    #[prop(optional_no_strip)] on_more: Option<Callback<()>>,
    /// A platform trigger for the next page, rendered below the button.
    #[prop(optional_no_strip)]
    auto_more: Option<ChildrenFn>,
    /// Said instead of the list when there is nothing to show and nothing coming.
    #[prop(optional, into)]
    empty: Option<String>,
    /// Whether this list paints its own empty note. A page with a richer empty state (a
    /// card, a spinner, an error) turns this off and paints it above the list.
    #[prop(default = true)]
    show_empty: bool,
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
                if !show_empty {
                    return ().into_any();
                }
                view! {
                    <p class="text-sm opacity-60">
                        {move || {
                            note.try_with_value(Clone::clone)
                                .flatten()
                                .unwrap_or_else(|| "Nothing here yet.".to_owned())
                        }}
                    </p>
                }
                .into_any()
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
        {auto_more.map(|more| more())}
    }
}

/// The composer and the tab strip as the server paints them before wasm arrives.
///
/// Inert on purpose: the wasm mount replaces this tree before the reader can type, and a
/// control that pretends to work is worse than one that plainly waits.
/// A column of already-painted cards, with the server's cursor button.
///
/// `cards` is the output of [`crate::status_card::StatusCard`], one per row: the server has
/// the data but not the wasm, so it paints the rows first and wraps them in the same column
/// the mounted client uses. When there is no cursor the button is absent rather than inert.
#[component]
pub fn StatusColumn(
    #[prop(into)] cards: String,
    #[prop(optional, into)] cursor: Option<String>,
) -> impl IntoView {
    view! {
        <ul class="flex flex-col gap-2" inner_html=cards></ul>
        {cursor.map(|cursor| {
            view! {
                <button class="btn btn-ghost btn-sm w-full" data-cursor=cursor>
                    "Load more"
                </button>
            }
        })}
    }
}
