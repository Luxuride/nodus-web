//! Shortcodes inside a post's text, drawn as the images they stand for.
//!
//! Mastodon hands a status its own `emojis` list — the custom emoji that appear in *this*
//! text. Anything outside that list is left as the writer typed it: a colon-word-colon in
//! prose ("the ratio is 3:1:0") must not turn into a broken image, and only the instance's
//! own knowledge decides what a shortcode means.

use leptos::prelude::*;
use nodus_instance::model::CustomEmoji;

/// One run of text, or one emoji, in the order the author wrote it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Piece {
    /// Text that is not a known shortcode.
    Text(String),
    /// A shortcode that resolves to an image.
    Emoji { shortcode: String, url: String },
}

/// Splits `text` into runs of prose and the shortcodes `emojis` resolves.
///
/// Kept as a plain function so the rule can be tested without a browser: the renderer is
/// then a `For` over whatever this returns.
#[must_use]
pub fn split(text: &str, emojis: &[CustomEmoji]) -> Vec<Piece> {
    let mut pieces = Vec::new();
    let mut rest = text;
    let mut prose = String::new();

    while let Some(open) = rest.find(':') {
        let after = &rest[open + 1..];
        let Some(close) = after.find(':') else {
            break;
        };
        let candidate = &after[..close];
        let known = emojis
            .iter()
            .find(|emoji| emoji.shortcode == candidate)
            .filter(|_| is_shortcode(candidate));

        if let Some(emoji) = known {
            prose.push_str(&rest[..open]);
            if !prose.is_empty() {
                pieces.push(Piece::Text(std::mem::take(&mut prose)));
            }
            pieces.push(Piece::Emoji {
                shortcode: emoji.shortcode.clone(),
                url: if emoji.url.is_empty() {
                    emoji.static_url.clone().unwrap_or_default()
                } else {
                    emoji.url.clone()
                },
            });
            rest = &after[close + 1..];
        } else {
            // Not an emoji: keep the colon in the prose and continue looking after it,
            // otherwise one unmatched colon would hide every shortcode that follows.
            prose.push_str(&rest[..=open]);
            rest = after;
        }
    }

    prose.push_str(rest);
    if !prose.is_empty() {
        pieces.push(Piece::Text(prose));
    }
    pieces
}

/// A shortcode is letters, digits and underscores — the same rule the composer applies
/// while completing, so what can be typed is what can render.
fn is_shortcode(candidate: &str) -> bool {
    !candidate.is_empty()
        && candidate
            .chars()
            .all(|character| character.is_alphanumeric() || character == '_')
}

/// Text with its known shortcodes drawn as inline images.
#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "a Leptos component receives its props by value"
)]
pub fn EmojiText(text: String, emojis: Vec<CustomEmoji>) -> impl IntoView {
    let pieces = StoredValue::new(split(&text, &emojis));
    view! {
        <For each=move || pieces.get_value() key=|piece| match piece {
            Piece::Text(text) => format!("t:{text}"),
            Piece::Emoji { shortcode, .. } => format!("e:{shortcode}"),
        } let:piece>
            {match piece {
                Piece::Text(text) => text.into_any(),
                Piece::Emoji { shortcode, url } => view! {
                    <img
                        class="emoji inline-block h-[1.2em] w-auto align-[-0.2em]"
                        src=url
                        alt=format!(":{shortcode}:")
                        title=format!(":{shortcode}:")
                        loading="lazy"
                        decoding="async"
                    />
                }
                .into_any(),
            }}
        </For>
    }
}

#[cfg(test)]
mod tests {
    use super::{Piece, split};

    fn emoji(shortcode: &str) -> super::CustomEmoji {
        super::CustomEmoji {
            shortcode: shortcode.to_owned(),
            url: format!("https://example.test/{shortcode}.png"),
            static_url: None,
        }
    }

    #[test]
    fn a_known_shortcode_becomes_an_image_between_its_prose() {
        let pieces = split("hello :blob: world", &[emoji("blob")]);
        assert_eq!(
            pieces,
            vec![
                Piece::Text("hello ".to_owned()),
                Piece::Emoji {
                    shortcode: "blob".to_owned(),
                    url: "https://example.test/blob.png".to_owned()
                },
                Piece::Text(" world".to_owned())
            ]
        );
    }

    #[test]
    fn an_unknown_shortcode_stays_text_and_does_not_swallow_the_next_one() {
        let pieces = split("3:1:0 and :blob:", &[emoji("blob")]);
        assert_eq!(
            pieces,
            vec![
                Piece::Text("3:1:0 and ".to_owned()),
                Piece::Emoji {
                    shortcode: "blob".to_owned(),
                    url: "https://example.test/blob.png".to_owned()
                }
            ]
        );
    }

    #[test]
    fn a_lone_colon_is_left_alone() {
        assert_eq!(
            split("wait:", &[emoji("blob")]),
            vec![Piece::Text("wait:".to_owned())]
        );
    }
}
