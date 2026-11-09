//! Emoji for the composer: shortcode suggestions while typing.
//!
//! Both the suggestions and the picker draw on the instance's own catalogue
//! (`/api/v1/custom_emojis`), which is the set Mastodon has cached — its own emoji plus
//! every federated emoji it has seen. Offering anything else would be offering a shortcode
//! that arrives as plain text. The catalogue itself is fetched by the caller: this module
//! only decides which emoji a query shows and how one is drawn.

use leptos::prelude::*;
use nodus_instance::model::CustomEmoji;

/// The emoji worth showing for a query, best match first.
///
/// An exact shortcode wins, then a prefix, then a substring, because a writer typing four
/// characters of a long name expects the emoji they began to spell rather than the first
/// one that happens to contain those letters.
#[must_use]
pub fn suggest<'a>(
    catalogue: &'a [CustomEmoji],
    query: &str,
    limit: usize,
) -> Vec<&'a CustomEmoji> {
    let mut exact = Vec::new();
    let mut prefix = Vec::new();
    let mut inside = Vec::new();
    for emoji in catalogue {
        let name = emoji.shortcode.to_lowercase();
        if query.is_empty() || name == query {
            exact.push(emoji);
        } else if name.starts_with(query) {
            prefix.push(emoji);
        } else if name.contains(query) {
            inside.push(emoji);
        }
    }
    exact
        .into_iter()
        .chain(prefix)
        .chain(inside)
        .take(limit)
        .collect()
}

/// One emoji, as a button that inserts `:shortcode:`.
#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "a Leptos component receives its props by value"
)]
pub fn EmojiButton(
    #[prop(into)] emoji: CustomEmoji,
    /// Inserts the shortcode. Without one the button is disabled, not missing.
    insert: Option<Callback<String>>,
) -> impl IntoView {
    let shortcode = emoji.shortcode.clone();
    let label = format!(":{}:", emoji.shortcode);
    let url = if emoji.url.is_empty() {
        emoji.static_url.clone().unwrap_or_default()
    } else {
        emoji.url.clone()
    };
    view! {
        <button
            type="button"
            class="btn btn-ghost btn-sm h-9 w-9 p-0"
            title=label.clone()
            aria-label=label.clone()
            disabled=move || insert.is_none()
            on:click=move |_| {
                if let Some(insert) = insert {
                    insert.run(label.clone());
                }
            }
        >
            <img class="h-5 w-5" src=url alt=shortcode.clone() loading="lazy" decoding="async"/>
        </button>
    }
}

/// The suggestion list above the textarea.
///
/// Rendered as a list rather than a dialog: the writer is typing, and a click on one entry
/// must not take focus away from the box they are in.
#[component]
pub fn Suggestions(
    /// The `:shortcode` being typed, without its colon.
    #[prop(into)]
    query: Signal<String>,
    /// Whether a token is open at all.
    #[prop(into)]
    open: Signal<bool>,
    #[prop(into)] catalogue: Signal<Vec<CustomEmoji>>,
    /// Inserts the shortcode.
    #[prop(optional)]
    insert: Option<Callback<String>>,
) -> impl IntoView {
    let matches = move || {
        suggest(&catalogue.get(), &query.get(), 8)
            .into_iter()
            .cloned()
            .collect::<Vec<_>>()
    };
    view! {
        <Show when=move || open.get() && !matches().is_empty()>
            <ul class="menu menu-horizontal flex-wrap gap-1 rounded-box bg-base-100 p-1 shadow">
                <For each=matches key=|emoji| emoji.shortcode.clone() let:emoji>
                    <li>
                        <EmojiButton emoji=emoji.clone() insert=insert/>
                    </li>
                </For>
            </ul>
        </Show>
    }
}
