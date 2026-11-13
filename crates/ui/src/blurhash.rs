//! Blurhash placeholders.
//!
//! A status carries a short blurhash string. The `blurhash` crate decodes it to RGBA,
//! `image` encodes a tiny PNG, and the browser scales that PNG behind the real file.
//! Server and browser use the same path, so the placeholder does not change on mount.

use base64::Engine as _;
use image::{ImageBuffer, ImageFormat, RgbaImage};
use std::io::Cursor;

const PLACEHOLDER_SIZE: u32 = 32;

/// A CSS `background-image` value for a blurhash placeholder, or `None` when the
/// string is absent or malformed.
#[must_use]
pub fn placeholder(hash: &str) -> Option<String> {
    let pixels = blurhash::decode(hash, PLACEHOLDER_SIZE, PLACEHOLDER_SIZE, 1.0).ok()?;
    let image: RgbaImage = ImageBuffer::from_raw(PLACEHOLDER_SIZE, PLACEHOLDER_SIZE, pixels)?;
    let mut png = Cursor::new(Vec::new());
    image.write_to(&mut png, ImageFormat::Png).ok()?;
    let encoded = base64::engine::general_purpose::STANDARD.encode(png.into_inner());
    Some(format!("url(\"data:image/png;base64,{encoded}\")"))
}

/// The inline background declaration for one tile.
#[must_use]
pub fn placeholder_style(hash: &str) -> String {
    placeholder(hash).map_or_else(String::new, |image| {
        format!(
            "background-image: {image}; background-size: cover; \
             background-position: center;"
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_known_hash_becomes_a_png_data_uri() {
        let style = placeholder_style("LEHV6nWB2yk8pyo0adR*.7kCMdnj");
        assert!(style.starts_with("background-image: url(\"data:image/png;base64,iVBOR"));
        assert!(style.contains("background-size: cover"));
    }

    #[test]
    fn malformed_input_is_none_not_a_panic() {
        assert!(placeholder("").is_none());
        assert!(placeholder("!!!!!!").is_none());
    }
}
