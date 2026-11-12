//! A deliberately small MFM renderer.
//!
//! Misskey sends note text as MFM; Mastodon-shaped clients receive HTML. The
//! shared body renders the former only when the status says its source is MFM.
//! This is not a full MFM implementation: it covers the formatting and the
//! animation functions the timeline uses, escapes every text and attribute, and
//! leaves unknown functions visible as their content rather than dropping it.

use nodus_client_core::model::CustomEmoji;

/// Renders MFM to HTML, substituting the status's custom emoji.
#[must_use]
pub fn render(source: &str, emojis: &[CustomEmoji]) -> String {
    paint_emojis(inline(source), emojis)
}

fn inline(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut rest = source;
    while !rest.is_empty() {
        if let Some((body, after)) = delimited(rest, "**", "**") {
            out.push_str("<strong>");
            out.push_str(&inline(body));
            out.push_str("</strong>");
            rest = after;
        } else if let Some((body, after)) = delimited(rest, "~~", "~~") {
            out.push_str("<del>");
            out.push_str(&inline(body));
            out.push_str("</del>");
            rest = after;
        } else if let Some((body, after)) = delimited(rest, "*", "*") {
            out.push_str("<em>");
            out.push_str(&inline(body));
            out.push_str("</em>");
            rest = after;
        } else if let Some((body, after)) = delimited(rest, "`", "`") {
            out.push_str("<code>");
            out.push_str(&escape(body));
            out.push_str("</code>");
            rest = after;
        } else if let Some(after) = rest.strip_prefix("$[") {
            if let Some((body, after)) = bracket(after) {
                out.push_str(&function(body));
                rest = after;
            } else {
                out.push_str("$[");
                rest = after;
            }
        } else {
            let character = rest.chars().next().expect("non-empty");
            match character {
                '\n' => out.push_str("<br>"),
                '&' => out.push_str("&amp;"),
                '<' => out.push_str("&lt;"),
                '>' => out.push_str("&gt;"),
                '"' => out.push_str("&quot;"),
                '\'' => out.push_str("&#39;"),
                _ => out.push(character),
            }
            rest = &rest[character.len_utf8()..];
        }
    }
    out
}

/// Returns the body and the remainder when `source` starts with `open` and has
/// a matching `close` later. An empty body is accepted but an empty delimiter
/// pair is not, so `**` alone stays literal.
fn delimited<'a>(source: &'a str, open: &str, close: &str) -> Option<(&'a str, &'a str)> {
    let after_open = source.strip_prefix(open)?;
    let end = after_open.find(close)?;
    if end == 0 {
        return None;
    }
    Some((&after_open[..end], &after_open[end + close.len()..]))
}

/// Finds the closing `]` of a function body, respecting nested brackets.
fn bracket(source: &str) -> Option<(&str, &str)> {
    let mut depth = 0_usize;
    for (index, character) in source.char_indices() {
        match character {
            '[' => depth += 1,
            ']' if depth == 0 => return Some((&source[..index], &source[index + 1..])),
            ']' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    None
}

fn function(body: &str) -> String {
    let (head, content) = body.split_once(' ').unwrap_or((body, ""));
    let (name, argument) = head
        .split_once('.')
        .map_or((head, None), |(name, argument)| (name, Some(argument)));
    match name {
        "x2" | "x3" | "x4" | "spin" | "shake" | "jelly" | "tada" | "jump" | "bounce"
        | "rainbow" | "sparkle" | "blur" => {
            format!("<span class=\"mfm-{name}\">{}</span>", inline(content))
        }
        "font" if matches!(argument, Some("serif" | "sans" | "monospace" | "cursive" | "fantasy")) => {
            format!(
                "<span class=\"mfm-font\" style=\"font-family:{}\">{}</span>",
                argument.unwrap_or_default(),
                inline(content)
            )
        }
        "fg" | "bg" if argument.is_some_and(safe_color) => {
            let property = if name == "fg" { "color" } else { "background-color" };
            format!(
                "<span style=\"{property}:{}\">{}</span>",
                argument.unwrap_or_default(),
                inline(content)
            )
        }
        _ => inline(content),
    }
}

fn safe_color(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'#' | b'%'))
}

fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(character),
        }
    }
    out
}

fn paint_emojis(mut html: String, emojis: &[CustomEmoji]) -> String {
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
        let source = escape(source);
        let replacement = format!(
            "<img class=\"emoji inline-block h-[1.2em] w-auto align-[-0.2em]\" alt=\"{token}\" title=\"{token}\" loading=\"lazy\" decoding=\"async\" src=\"{source}\">",
            token = escape(&token),
        );
        html = html.replace(&token, &replacement);
    }
    html
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_is_escaped_before_markup_is_added() {
        assert_eq!(inline("<script>&"), "&lt;script&gt;&amp;");
    }

    #[test]
    fn emphasis_and_code_are_rendered() {
        assert_eq!(
            inline("**bold** *italic* ~~gone~~ `x<y`"),
            "<strong>bold</strong> <em>italic</em> <del>gone</del> <code>x&lt;y</code>"
        );
    }

    #[test]
    fn animation_functions_wrap_their_content() {
        assert_eq!(inline("$[spin hello]"), "<span class=\"mfm-spin\">hello</span>");
    }

    #[test]
    fn unknown_functions_keep_their_content() {
        assert_eq!(inline("$[unknown hello]"), "hello");
    }

    #[test]
    fn unsafe_colours_are_ignored() {
        assert_eq!(inline("$[fg.red;x hello]"), "hello");
    }

    #[test]
    fn custom_emoji_are_painted_once() {
        let emoji = CustomEmoji {
            shortcode: "wave".to_owned(),
            url: "https://cdn.test/wave.png".to_owned(),
            static_url: None,
        };
        let html = render("hi :wave:", &[emoji]);
        assert_eq!(
            html.matches("src=\"https://cdn.test/wave.png\"").count(),
            1,
            "html was {html}"
        );
    }
}
