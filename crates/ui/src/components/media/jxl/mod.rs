//! JPEG XL fallback for browsers that cannot display it themselves.
//!
//! Browsers that support `image/jxl` get the original URL. Everywhere else the file is
//! fetched and decoded with `jxl-oxide` in WebAssembly, then handed back as a PNG data
//! URI, so the rest of the media grid stays format-agnostic.

#[cfg(target_arch = "wasm32")]
use base64::Engine as _;
#[cfg(target_arch = "wasm32")]
use std::io::Cursor;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;

/// Whether a URL names a JPEG XL file, ignoring a query string.
#[must_use]
pub fn is_jxl(url: &str) -> bool {
    url.split(['?', '#'])
        .next()
        .is_some_and(|path| path.to_ascii_lowercase().ends_with(".jxl"))
}

/// The browser's own JPEG XL decoder, when it advertises one.
#[cfg(target_arch = "wasm32")]
pub async fn browser_supports_jxl() -> bool {
    use wasm_bindgen::JsValue;

    let Some(window) = web_sys::window() else {
        return false;
    };
    let Ok(decoder) = js_sys::Reflect::get(&window, &JsValue::from_str("ImageDecoder")) else {
        return false;
    };
    let Ok(is_supported) = js_sys::Reflect::get(&decoder, &JsValue::from_str("isTypeSupported"))
    else {
        return false;
    };
    let Ok(function) = is_supported.dyn_into::<js_sys::Function>() else {
        return false;
    };
    let Ok(result) = function.call1(&decoder, &JsValue::from_str("image/jxl")) else {
        return false;
    };
    let Ok(promise) = result.dyn_into::<js_sys::Promise>() else {
        return false;
    };
    JsFuture::from(promise)
        .await
        .is_ok_and(|value| value.as_bool() == Some(true))
}

/// Fetch a JPEG XL file and return it as a PNG data URI.
#[cfg(target_arch = "wasm32")]
pub async fn decode_to_png(url: &str) -> Option<String> {
    use image::{ImageBuffer, ImageFormat};

    let window = web_sys::window()?;
    let response = JsFuture::from(window.fetch_with_str(url)).await.ok()?;
    let response: web_sys::Response = response.dyn_into().ok()?;
    if !response.ok() {
        return None;
    }
    let buffer = JsFuture::from(response.array_buffer().ok()?).await.ok()?;
    let bytes = js_sys::Uint8Array::new(&buffer).to_vec();

    let image = jxl_oxide::JxlImage::builder()
        .read(Cursor::new(bytes))
        .ok()?;
    let render = image.render_frame(0).ok()?;
    let mut stream = render.stream();
    let (width, height) = (stream.width(), stream.height());
    let channels = stream.channels() as usize;
    if width == 0 || height == 0 || !(3..=4).contains(&channels) {
        return None;
    }
    let mut samples = vec![0_u8; width as usize * height as usize * channels];
    stream.write_to_buffer(&mut samples);
    let mut rgba = Vec::with_capacity(width as usize * height as usize * 4);
    for pixel in samples.chunks_exact(channels) {
        rgba.extend_from_slice(&pixel[..3]);
        rgba.push(if channels == 4 { pixel[3] } else { 255 });
    }
    let image: image::RgbaImage = ImageBuffer::from_raw(width, height, rgba)?;
    let mut png = Cursor::new(Vec::new());
    image.write_to(&mut png, ImageFormat::Png).ok()?;
    let encoded = base64::engine::general_purpose::STANDARD.encode(png.into_inner());
    Some(format!("data:image/png;base64,{encoded}"))
}

#[cfg(test)]
mod tests {
    use super::is_jxl;

    #[test]
    fn only_the_extension_counts() {
        assert!(is_jxl("https://cdn.test/a.jxl"));
        assert!(is_jxl("https://cdn.test/a.JXL?size=large"));
        assert!(!is_jxl("https://cdn.test/a.jpg"));
        assert!(!is_jxl("https://cdn.test/a.jxl.jpg"));
    }
}
