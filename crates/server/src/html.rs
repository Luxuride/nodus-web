//! Mastodon hands out HTML, and a server that prints it as text prints `<br>`.
//!
//! The instance is the only author of these fragments, but "the instance is trusted" is
//! the wrong reason to interpolate them into a page: a compromised or misbehaving server
//! would then own this origin. `lol-html` rewrites the fragment with an allowlist, so the
//! result is markup this server chose to keep rather than markup it passed through.
//!
//! What survives is what a post is made of: paragraphs, breaks, emphasis, links, lists,
//! quotes and inline images (custom emoji). Everything else goes, and so does every
//! attribute that is not in [`allowed`] — which is what removes `style`, `onclick` and the
//! dozens of `class` values Mastodon uses for its own layout.

use lol_html::{RewriteStrSettings, element, rewrite_str};

/// Elements a post may keep. Anything not listed is removed together with its content,
/// so a `<script>` cannot survive as text either.
const KEEP: &[&str] = &[
    "p", "br", "a", "span", "strong", "em", "b", "i", "u", "s", "del", "ins", "code", "pre",
    "blockquote", "ul", "ol", "li", "h1", "h2", "h3", "h4", "img",
];

/// The attributes worth keeping, per element.
fn allowed(tag: &str, name: &str) -> bool {
    matches!(
        (tag, name),
        ("a", "href" | "class" | "rel" | "target" | "title")
            | ("img", "src" | "alt" | "class" | "title" | "width" | "height")
            | ("span" | "p" | "code" | "pre" | "blockquote" | "ul" | "ol" | "li", "class")
    )
}

/// Whether a URL is one this page is willing to link to.
fn safe_url(url: &str) -> bool {
    let trimmed = url.trim_start();
    let lower = trimmed.to_ascii_lowercase();
    lower.starts_with("https://") || lower.starts_with("http://") || lower.starts_with('/')
}

/// Rewrites a status body into markup this origin is willing to serve.
///
/// On a rewrite error the escaped text is returned: a post that cannot be sanitised is
/// still readable, and reading beats a blank card.
#[must_use]
pub fn body(html: &str) -> String {
    let settings = RewriteStrSettings {
        element_content_handlers: vec![element!("*", |el| {
            let tag = el.tag_name();
            if !KEEP.contains(&tag.as_str()) {
                el.remove();
                return Ok(());
            }

            // Collected first: removing while iterating borrows the same element twice.
            let doomed: Vec<String> = el
                .attributes()
                .iter()
                .filter(|attribute| !allowed(&tag, &attribute.name()))
                .map(|attribute| attribute.name().clone())
                .collect();
            for name in doomed {
                el.remove_attribute(&name);
            }

            if tag == "a" {
                let href = el.get_attribute("href").unwrap_or_default();
                if href.is_empty() || !safe_url(&href) {
                    el.remove_attribute("href");
                } else {
                    // A link out of this page opens in its own tab, and says so.
                    let _ = el.set_attribute("rel", "nofollow noopener noreferrer");
                    let _ = el.set_attribute("target", "_blank");
                }
            }
            if tag == "img" {
                let src = el.get_attribute("src").unwrap_or_default();
                if src.is_empty() || !safe_url(&src) {
                    el.remove();
                    return Ok(());
                }
                let _ = el.set_attribute("loading", "lazy");
                let _ = el.set_attribute("referrerpolicy", "no-referrer");
                if el.get_attribute("alt").is_none() {
                    let _ = el.set_attribute("alt", "");
                }
            }
            Ok(())
        })],
        ..RewriteStrSettings::default()
    };

    rewrite_str(html, settings).unwrap_or_else(|_| {
        html.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
    })
}

#[cfg(test)]
mod tests {
    use super::body;

    #[test]
    fn paragraphs_and_breaks_survive() {
        let html = body("<p>one</p><p>two<br />three</p>");
        assert!(html.contains("<p>one</p>"), "{html}");
        assert!(html.contains("<br"), "{html}");
    }

    #[test]
    fn links_get_rel_and_target() {
        let html = body("<p><a href=\"https://example.org/x\">x</a></p>");
        assert!(html.contains("href=\"https://example.org/x\""), "{html}");
        assert!(html.contains("rel=\"nofollow noopener noreferrer\""), "{html}");
        assert!(html.contains("target=\"_blank\""), "{html}");
    }

    #[test]
    fn javascript_urls_lose_their_href() {
        let html = body("<a href=\"javascript:alert(1)\">x</a>");
        assert!(!html.contains("javascript:"), "{html}");
    }

    #[test]
    fn scripts_and_handlers_are_removed() {
        let html = body("<p onclick=\"steal()\">hi<script>alert(1)</script></p>");
        assert!(!html.contains("script"), "{html}");
        assert!(!html.contains("onclick"), "{html}");
        assert!(html.contains("hi"), "{html}");
    }

    #[test]
    fn inline_styles_are_dropped() {
        let html = body("<p style=\"position:fixed\">x</p>");
        assert!(!html.contains("style"), "{html}");
    }

    #[test]
    fn emoji_images_keep_their_source() {
        let html = body("<img src=\"https://example.org/e.png\" alt=\":blob:\">");
        assert!(html.contains("src=\"https://example.org/e.png\""), "{html}");
        assert!(html.contains("referrerpolicy=\"no-referrer\""), "{html}");
    }
}
