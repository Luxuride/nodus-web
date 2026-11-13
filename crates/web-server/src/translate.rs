//! Translation is a server concern: the provider key stays here, and the browser posts
//! one fragment it is already allowed to read.
//!
//! The request carries the body as the instance sent it. The reply is sanitised with the
//! same allow-list as the timeline, so a provider — or a compromised instance — cannot
//! introduce markup this origin does not already accept.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::App;
use crate::html;

#[derive(Debug, Deserialize)]
pub struct TranslateRequest {
    /// The post body, already HTML for a Mastodon status and plain text for MFM.
    pub text: String,
    /// The content warning, when the post has one.
    #[serde(default)]
    pub spoiler: Option<String>,
    /// The language the post declares, when it declares one.
    #[serde(default)]
    pub source: Option<String>,
    /// The reader's language.
    #[serde(default)]
    pub target: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TranslateResponse {
    pub content: String,
    pub spoiler: Option<String>,
    pub detected_language: Option<String>,
    pub provider: &'static str,
}

#[derive(Debug, Deserialize)]
struct ProviderResponse {
    #[serde(rename = "translatedText")]
    translated_text: String,
    #[serde(rename = "detectedLanguage")]
    detected_language: Option<DetectedLanguage>,
}

#[derive(Debug, Deserialize)]
struct DetectedLanguage {
    language: String,
}

/// Translate one status fragment and its warning.
pub async fn translate(
    State(_app): State<std::sync::Arc<App>>,
    Json(request): Json<TranslateRequest>,
) -> Response {
    let text = request.text.trim();
    if text.is_empty() {
        return error(StatusCode::BAD_REQUEST, "There is nothing to translate.");
    }
    if text.chars().count() > 10_000 {
        return error(
            StatusCode::PAYLOAD_TOO_LARGE,
            "That post is too long to translate.",
        );
    }
    let target = request
        .target
        .as_deref()
        .filter(|code| language_code(code))
        .unwrap_or("en");
    let source = request
        .source
        .as_deref()
        .filter(|code| language_code(code))
        .unwrap_or("auto");

    let Ok(endpoint) = std::env::var("NODUS_TRANSLATE_URL") else {
        return error(
            StatusCode::SERVICE_UNAVAILABLE,
            "Translation is not configured on this server.",
        );
    };
    let endpoint = endpoint.trim();
    if endpoint.is_empty() {
        return error(
            StatusCode::SERVICE_UNAVAILABLE,
            "Translation is not configured on this server.",
        );
    }

    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(12))
        .build()
    {
        Ok(client) => client,
        Err(build_error) => {
            tracing::error!(error = %build_error, "could not build translation client");
            return error(
                StatusCode::BAD_GATEWAY,
                "The translation service is unavailable.",
            );
        }
    };

    let (content, detected_language) =
        match call(&client, endpoint, text, source, target, "html").await {
            Ok((translated, detected)) => {
                let html = if translated.contains('<') {
                    translated
                } else {
                    plain_html(&translated)
                };
                (html::body(&html), detected)
            }
            Err(message) => return error(StatusCode::BAD_GATEWAY, &message),
        };
    let spoiler = match request.spoiler.as_deref().map(str::trim) {
        Some(spoiler) if !spoiler.is_empty() => {
            match call(&client, endpoint, spoiler, source, target, "text").await {
                Ok((translated, _)) => Some(translated),
                Err(_) => Some(spoiler.to_owned()),
            }
        }
        _ => None,
    };

    Json(TranslateResponse {
        content,
        spoiler,
        detected_language,
        provider: "libretranslate",
    })
    .into_response()
}

async fn call(
    client: &reqwest::Client,
    endpoint: &str,
    text: &str,
    source: &str,
    target: &str,
    format: &str,
) -> Result<(String, Option<String>), String> {
    let mut body = json!({
        "q": text,
        "source": source,
        "target": target,
        "format": format,
    });
    if let Ok(key) = std::env::var("NODUS_TRANSLATE_API_KEY")
        && !key.trim().is_empty()
    {
        body["api_key"] = json!(key);
    }
    let response = client
        .post(endpoint)
        .json(&body)
        .send()
        .await
        .map_err(|error| format!("The translation service did not answer: {error}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "The translation service rejected the request ({}).",
            response.status()
        ));
    }
    let answer: ProviderResponse = response
        .json()
        .await
        .map_err(|error| format!("The translation service sent an unreadable answer: {error}"))?;
    if answer.translated_text.trim().is_empty() {
        return Err("The translation service returned an empty translation.".to_owned());
    }
    Ok((
        answer.translated_text.trim().to_owned(),
        answer
            .detected_language
            .map(|detected| detected.language)
            .filter(|language| !language.trim().is_empty()),
    ))
}

fn plain_html(text: &str) -> String {
    let escaped = text
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    format!("<p>{}</p>", escaped.replace('\n', "<br>"))
}

fn language_code(code: &str) -> bool {
    let mut parts = code.split('-');
    let Some(primary) = parts.next() else {
        return false;
    };
    (2..=3).contains(&primary.len())
        && primary
            .chars()
            .all(|character| character.is_ascii_alphabetic())
        && parts.all(|part| {
            (2..=8).contains(&part.len())
                && part
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric())
        })
}

fn error(status: StatusCode, message: &str) -> Response {
    (status, Json(json!({ "error": message }))).into_response()
}

#[cfg(test)]
mod tests {
    use super::{language_code, plain_html};

    #[test]
    fn language_codes_are_narrow() {
        assert!(language_code("en"));
        assert!(language_code("pt-BR"));
        assert!(!language_code("en; DROP TABLE"));
        assert!(!language_code(""));
    }

    #[test]
    fn plain_text_becomes_paragraphs_without_markup() {
        assert_eq!(plain_html("one\ntwo < 3"), "<p>one<br>two &lt; 3</p>");
    }
}
