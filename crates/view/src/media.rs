//! The media rules both paints obey.
//!
//! A tile reserves its box before the file arrives, and a hidden attachment is veiled rather
//! than removed. Those two rules decide where the rest of a card sits, so they cannot be
//! re-derived per paint: the server would shift the page when the image landed, and the
//! client would shift it when the veil came off.

use nodus_instance::model::{MediaAttachment, MediaKind, Status};

/// The box a tile reserves: published dimensions when the instance has them, 16:9 when it
/// does not.
///
/// `None` means "the wire carried no usable size", which is different from "1600×900": the
/// first is an aspect ratio the box keeps, the second is a `width`/`height` pair the browser
/// can use for layout before the file arrives.
#[must_use]
pub fn extent(attachment: &MediaAttachment) -> (Option<(u32, u32)>, String) {
    match (attachment.meta_width, attachment.meta_height) {
        (Some(width), Some(height)) if width > 0 && height > 0 => {
            (Some((width, height)), String::new())
        }
        _ => (None, "aspect-ratio: 16 / 9;".to_owned()),
    }
}

/// The inline style that reserves a tile's box.
#[must_use]
pub fn box_style(attachment: &MediaAttachment) -> String {
    let (pair, fallback) = extent(attachment);
    match pair {
        Some((width, height)) => format!("{fallback}aspect-ratio: {width} / {height};"),
        None => fallback,
    }
}

/// The veil over a hidden attachment.
///
/// The scale is not decoration: a blur samples outside the element and would otherwise eat
/// its own edges, leaving a bright rim that shows the picture's outline.
#[must_use]
pub fn veil(blurred: bool) -> &'static str {
    if blurred {
        "filter: blur(18px); transform: scale(1.08);"
    } else {
        ""
    }
}

/// "Show 1 attachment" or "Show 3 attachments", worded once.
#[must_use]
pub fn attachments_label(status: &Status) -> String {
    match status.media_attachments.len() {
        1 => "Show 1 attachment".to_owned(),
        count => format!("Show {count} attachments"),
    }
}

/// Whether a player element (rather than an image) is the right tile for this kind.
#[must_use]
pub fn is_player(kind: MediaKind) -> bool {
    matches!(kind, MediaKind::Video | MediaKind::Gifv | MediaKind::Audio)
}

/// The inline style for a tile whose size may be unknown, which is what a Leptos view has
/// in hand before it has an attachment.
#[must_use]
pub fn style_for(width: Option<u32>, height: Option<u32>) -> String {
    match (width, height) {
        (Some(width), Some(height)) if width > 0 && height > 0 => {
            format!("aspect-ratio: {width} / {height};")
        }
        _ => "aspect-ratio: 16 / 9;".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attachment(width: Option<u32>, height: Option<u32>) -> MediaAttachment {
        MediaAttachment {
            id: "1".to_owned(),
            kind: MediaKind::Image,
            url: Some("https://example.test/a.png".to_owned()),
            preview_url: None,
            description: None,
            blurhash: None,
            meta_width: width,
            meta_height: height,
        }
    }

    #[test]
    fn a_published_size_is_used_and_an_absent_one_falls_back_to_sixteen_by_nine() {
        assert_eq!(
            box_style(&attachment(Some(800), Some(600))),
            "aspect-ratio: 800 / 600;"
        );
        assert_eq!(box_style(&attachment(None, None)), "aspect-ratio: 16 / 9;");
        assert_eq!(
            box_style(&attachment(Some(0), Some(600))),
            "aspect-ratio: 16 / 9;"
        );
    }

    #[test]
    fn a_veiled_tile_scales_so_the_blur_has_no_rim() {
        assert!(veil(true).contains("scale(1.08)"));
        assert_eq!(veil(false), "");
    }
}
