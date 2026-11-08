//! The tile geometry, read from the shared rules rather than restated here.
//!
//! A card's height must not change when a picture arrives, on either paint. `nodus_view`
//! owns that rule; this module is only the adapter from the two numbers a Leptos view has
//! in hand to the style string the box needs.

/// The inline style that reserves a media tile's box.
#[must_use]
pub fn style(width: Option<u32>, height: Option<u32>) -> String {
    let attachment = nodus_instance::model::MediaAttachment {
        id: String::new(),
        kind: nodus_instance::model::MediaKind::Image,
        url: None,
        preview_url: None,
        description: None,
        blurhash: None,
        meta_width: width,
        meta_height: height,
    };
    nodus_view::media::box_style(&attachment)
}
