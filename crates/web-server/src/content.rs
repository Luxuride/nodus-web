//! What a status body becomes before it is painted.
//!
//! The instance hands over HTML with mentions and hashtags already resolved, and custom emoji
//! as `<img class="emoji" alt=":code:">`. Two things still have to happen before it may be
//! embedded: the markup is sanitized ([`crate::html::body`]) and any remaining `:code:` that
//! the status itself declares is replaced. Both paints end up with the same string, which is
//! why this is one function rather than a line at each call site.

use crate::document::escape;

/// Sanitises a status body and paints the emoji the status itself declares.
pub fn prepared(content: &str, emojis: &[nodus_instance::model::CustomEmoji]) -> String {
    with_emojis(crate::html::body(content), emojis)
}

fn with_emojis(html: String, emojis: &[nodus_instance::model::CustomEmoji]) -> String {
    if emojis.is_empty() {
        return html;
    }
    let mut painted = html;
    for emoji in emojis {
        let source = if emoji.url.is_empty() {
            emoji.static_url.as_deref().unwrap_or_default()
        } else {
            emoji.url.as_str()
        };
        if source.is_empty() {
            continue;
        }
        let token = format!(":{}:", emoji.shortcode);
        let replacement = format!(
            "<img class=\"emoji inline-block h-[1.2em] w-auto align-[-0.2em]\" alt=\"{token}\" title=\"{token}\" loading=\"lazy\" decoding=\"async\" src=\"{source}\">",
            token = escape(&token),
            source = escape(source)
        );
        painted = painted.replace(&token, &replacement);
    }
    painted
}
