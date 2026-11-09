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
//!
//! What is left here is the part that is not markup: the token scanner, the fetch, and the
//! text edit that replaces a pending shortcode. The buttons and the suggestion list are the
//! view's ([`Suggestions`] is re-exported from there), because they are drawn the same way
//! wherever a composer appears.

use crate::state::bridge::actions;
use leptos::prelude::*;
use nodus_instance::model::CustomEmoji;

pub use nodus_pages::pages::compose::inline::emoji::Suggestions;

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
