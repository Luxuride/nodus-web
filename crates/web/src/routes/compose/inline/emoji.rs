//! Emoji for the composer: shortcode suggestions while typing, and a picker.
#![allow(
    dead_code,
    reason = "the controls are wired into the textarea in the next step"
)]
//!
//! Both sources are the instance's own catalogue (`/api/v1/custom_emojis`), which is the
//! set Mastodon has cached — its own emoji plus every federated emoji it has seen. That
//! matters for the picker *and* for the suggestion list: an emoji copied from another
//! server is only renderable if this instance knows it, so offering anything else would be
//! offering a shortcode that arrives as plain text.

use leptos::prelude::*;
use nodus_instance::model::CustomEmoji;

use crate::state::bridge::actions;

/// The `:shortcode` being typed at the end of `text`, with the byte index of its colon.
///
/// Only the token at the very end counts: the composer inserts at the caret, which is the
/// end of the string in every case the inline box supports, and searching the whole text
/// would pop a list while the writer is editing an earlier sentence.
#[must_use]
pub fn pending(text: &str) -> Option<(usize, String)> {
    let colon = text.rfind(':')?;
    let query = &text[colon + 1..];
    // A shortcode is letters, digits and underscores; anything else (a space, a second
    // colon, a URL) means this colon is not the start of a shortcode.
    if query.contains(':') || query.contains(' ') || query.contains('\n') {
        return None;
    }
    if !query.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return None;
    }
    Some((colon, query.to_lowercase()))
}

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
pub fn EmojiButton(emoji: CustomEmoji, insert: Callback<String>) -> impl IntoView {
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
            on:click=move |_| insert.run(label.clone())
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
    query: Signal<String>,
    open: ReadSignal<bool>,
    catalogue: ReadSignal<Vec<CustomEmoji>>,
    insert: Callback<String>,
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

/// The picker: the whole catalogue, searchable.
#[component]
pub fn Picker(
    open: ReadSignal<bool>,
    set_open: WriteSignal<bool>,
    catalogue: ReadSignal<Vec<CustomEmoji>>,
    insert: Callback<String>,
) -> impl IntoView {
    let (query, set_query) = signal(String::new());
    let matches = move || {
        let wanted = query.get().to_lowercase();
        let all = catalogue.get();
        if wanted.is_empty() {
            all
        } else {
            all.into_iter()
                .filter(|emoji| emoji.shortcode.to_lowercase().contains(&wanted))
                .collect()
        }
    };
    let close = move |()| {
        set_query.set(String::new());
        set_open.set(false);
    };
    view! {
        <Show when=move || open.get()>
            <div class="rounded-box border border-base-300 bg-base-100 p-2 shadow">
                <div class="flex items-center gap-2">
                    <input
                        class="input input-bordered input-sm w-full"
                        type="search"
                        placeholder="Search emoji"
                        prop:value=query
                        on:input=move |event| set_query.set(event_target_value(&event))
                    />
                    <button type="button" class="btn btn-ghost btn-sm" on:click=move |_| close(())>
                        "Close"
                    </button>
                </div>
                <div class="mt-2 flex max-h-48 flex-wrap gap-1 overflow-y-auto">
                    <For each=matches key=|emoji| emoji.shortcode.clone() let:emoji>
                        <EmojiButton emoji=emoji.clone() insert=insert/>
                    </For>
                </div>
            </div>
        </Show>
    }
}

/// The catalogue, fetched the first time a composer needs it.
///
/// A signal rather than a local variable because the picker and the suggestion list read
/// the same list, and a second fetch would double the instance's largest metadata request.
pub fn catalogue() -> ReadSignal<Vec<CustomEmoji>> {
    let (emojis, set_emojis) = signal(Vec::<CustomEmoji>::new());
    leptos::task::spawn_local(async move {
        let all = actions::custom_emojis().await;
        set_emojis.set(all);
    });
    emojis
}

/// Replaces the pending shortcode in `text` with `:shortcode: `, and answers the new text.
///
/// The trailing space is deliberate: a writer who picks an emoji has finished that word.
#[must_use]
pub fn complete(text: &str, colon: usize, shortcode: &str) -> String {
    let mut finished = text[..colon].to_owned();
    finished.push_str(shortcode);
    finished.push(' ');
    finished
}
