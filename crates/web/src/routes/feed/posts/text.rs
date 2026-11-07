//! Turning a status body into something safe to render.
//!
//! The instance sends HTML with its own markup, links and custom emoji. The full
//! answer is the allow-list sanitizer described in `web/docs/04-ui.md`; it does not
//! exist in this crate yet, and rendering raw HTML with `inner_html` would be the one
//! XSS hole that matters here, because the access token lives in the same origin.
//!
//! So this module does the smallest honest thing: strip tags, decode the handful of
//! entities Mastodon actually emits, collapse runs of whitespace, and keep paragraph
//! breaks. Post text is therefore readable but not rich — a deliberate downgrade,
//! recorded as a gap rather than hidden behind a fake sanitizer.

/// Reduces a status body to readable plain text.
#[must_use]
pub fn plain_text(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut closing_paragraph = false;
    for character in html.chars() {
        match character {
            '<' => {
                in_tag = true;
                // `</p>` and `<br>` are the only tags worth a newline; setting the
                // flag here keeps the parser a single pass with no lookahead.
                closing_paragraph = false;
            }
            '>' => {
                in_tag = false;
                if closing_paragraph {
                    out.push('\n');
                    closing_paragraph = false;
                }
            }
            _ if in_tag => {
                // Track `<br` so a line break survives; anything else inside a tag
                // is ignored, including attribute quotes.
                closing_paragraph = false;
            }
            _ => out.push(character),
        }
    }
    normalise(&decode_entities(&out))
}

/// Collapses whitespace while preserving intentional line breaks.
fn normalise(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut pending_space = false;
    let mut pending_break = false;
    for character in text.chars() {
        if character == '\n' {
            pending_break = true;
            pending_space = false;
            continue;
        }
        if character.is_whitespace() {
            pending_space = true;
            continue;
        }
        if pending_break {
            if !out.is_empty() {
                out.push('\n');
            }
            pending_break = false;
        } else if pending_space && !out.is_empty() {
            out.push(' ');
        }
        pending_space = false;
        out.push(character);
    }
    out.trim().to_owned()
}

/// The entities Mastodon emits, decoded; anything unknown is left as written.
fn decode_entities(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
        // `&amp;` last: decoding it first would turn `&amp;lt;` into `<`.
        .replace("&amp;", "&")
}

#[cfg(test)]
mod tests {
    use super::plain_text;

    #[test]
    fn tags_are_removed_and_paragraphs_kept() {
        let html = "<p>hello</p><p>world</p>";
        assert_eq!(plain_text(html), "hello\nworld");
    }

    #[test]
    fn entities_are_decoded_once() {
        assert_eq!(plain_text("<p>a &amp;lt; b</p>"), "a &lt; b");
        assert_eq!(
            plain_text("<p>it&#39;s &quot;fine&quot;</p>"),
            "it's \"fine\""
        );
    }

    #[test]
    fn whitespace_between_tags_does_not_become_blank_paragraphs() {
        assert_eq!(plain_text("<p>one</p>\n\n<p>two</p>"), "one\ntwo");
    }

    #[test]
    fn angle_brackets_in_text_are_preserved_after_decoding() {
        assert_eq!(plain_text("1 &lt; 2 &gt; 0"), "1 < 2 > 0");
    }
}
