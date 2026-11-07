//! The three values PKCE needs: a verifier, its S256 challenge, and a `state`.
//!
//! Randomness and hashing come from the browser's Web Crypto API, so the flow adds
//! no dependency and no Rust cryptography to audit. `crypto.subtle` is only exposed
//! in a secure context — https, or http on localhost — which is also the only place
//! an OAuth redirect is worth trusting with a token.

use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;

/// RFC 7636 §4.1 requires at least 32 bytes of entropy for the verifier; 32 bytes
/// become 43 base64url characters without padding.
const VERIFIER_BYTES: usize = 32;

/// `state` is only compared, never derived from, so it needs to be unguessable and
/// nothing more. 16 bytes is the strength a session cookie would carry and keeps
/// the stored record short.
const STATE_BYTES: usize = 16;

/// A fresh verifier: 32 random bytes, base64url without padding.
pub(crate) fn verifier() -> Result<String, String> {
    Ok(base64url(&random(VERIFIER_BYTES)?))
}

/// A fresh `state` value. Base64url is also the RFC 7636 §4.1 character set, which
/// is what both a query parameter and the stored record want.
pub(crate) fn state() -> Result<String, String> {
    Ok(base64url(&random(STATE_BYTES)?))
}

/// `BASE64URL(SHA256(verifier))`, the S256 challenge of RFC 7636 §4.2.
///
/// S256 rather than `plain` because the whole point of PKCE is that an observer who
/// sees the authorization request cannot use the challenge as the verifier.
pub(crate) async fn challenge(verifier: &str) -> Result<String, String> {
    let promise = crypto()?
        .subtle()
        .digest_with_str_and_u8_array("SHA-256", verifier.as_bytes())
        .map_err(|error| js_message(&error))?;
    let digest = JsFuture::from(promise)
        .await
        .map_err(|error| js_message(&error))?;
    // `digest` resolves to an `ArrayBuffer`; `Uint8Array` is the view that can be
    // copied out into Rust.
    Ok(base64url(&js_sys::Uint8Array::new(&digest).to_vec()))
}

/// Fills `len` bytes from `crypto.getRandomValues`.
///
/// There is deliberately no `Math.random` fallback: a predictable verifier makes
/// PKCE worthless, so an unavailable Web Crypto is a reason to stop, not to degrade.
fn random(len: usize) -> Result<Vec<u8>, String> {
    let mut bytes = vec![0_u8; len];
    crypto()?
        .get_random_values_with_u8_array(&mut bytes)
        .map_err(|error| js_message(&error))?;
    Ok(bytes)
}

/// The page's `Crypto` object, or a message explaining why sign-in cannot proceed.
fn crypto() -> Result<web_sys::Crypto, String> {
    web_sys::window()
        .and_then(|window| window.crypto().ok())
        .ok_or_else(|| {
            "This browser does not expose Web Crypto, which sign-in needs. Serve the app over \
             https (or localhost)."
                .to_owned()
        })
}

/// A JavaScript exception as a sentence.
///
/// `JsValue`'s `Debug` prints the JS type, not the `message` of a `DOMException`,
/// so the property is read explicitly before giving up.
fn js_message(error: &JsValue) -> String {
    error.as_string().unwrap_or_else(|| {
        js_sys::Reflect::get(error, &JsValue::from_str("message"))
            .ok()
            .and_then(|message| message.as_string())
            .unwrap_or_else(|| "the browser reported an unknown error".to_owned())
    })
}

/// Base64url without padding (RFC 4648 §5).
///
/// Hand-written on purpose: it is twenty lines, and a crate for it would be larger
/// than the code. The `-_` alphabet and the absence of `=` are not cosmetic — they
/// are what makes the value safe in a URL query and in a form body unchanged.
fn base64url(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        // A missing trailing byte contributes zero bits, which is exactly why
        // padding can be dropped without losing information.
        let b0 = chunk[0];
        let b1 = chunk.get(1).copied().unwrap_or(0);
        let b2 = chunk.get(2).copied().unwrap_or(0);
        out.push(char::from(ALPHABET[usize::from(b0 >> 2)]));
        out.push(char::from(ALPHABET[usize::from(((b0 & 0x03) << 4) | (b1 >> 4))]));
        if chunk.len() > 1 {
            out.push(char::from(ALPHABET[usize::from(((b1 & 0x0f) << 2) | (b2 >> 6))]));
        }
        if chunk.len() > 2 {
            out.push(char::from(ALPHABET[usize::from(b2 & 0x3f)]));
        }
    }
    out
}
