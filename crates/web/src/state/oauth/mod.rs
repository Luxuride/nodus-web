//! Mastodon sign-in: OAuth 2.0 authorization-code flow with PKCE.
//!
//! The flow, in order — the same one Elk and Phanpy use:
//!
//! 1. register this client with the instance (`POST /api/v1/apps`). It is a public
//!    client, so the `client_secret` in the response is never read, stored or sent;
//! 2. send the browser to `/oauth/authorize` with an S256 challenge and a `state`;
//! 3. the instance redirects back to `/oauth/callback`, where [`finish`] checks
//!    `state` and exchanges the code for a token with the verifier.
//!
//! Two properties are deliberate:
//!
//! * **No secret anywhere.** PKCE replaces the client secret, and the verifier never
//!   leaves this browser, so a leaked `client_id` buys an attacker nothing.
//! * **The pending record lives in `sessionStorage`.** It is per-tab and dies with
//!   the tab, which is exactly the lifetime of a half-finished authorization; the
//!   resulting token goes to `localStorage` through `bridge::save_session`, as before.

use url::Url;

mod api;
mod pending;
mod pkce;

/// The scopes this client actually uses; must stay in step with the scopes the
/// bridge assumes when it restores a session (`state/bridge/session.rs`).
///
/// Mastodon accepts this space-separated form in the registration, the authorization
/// request and the token exchange. Asking for more would make the consent screen
/// worse without buying a feature.
pub(super) const SCOPES: &str = "read write follow";

/// Where the instance sends the browser back.
///
/// The same string is registered as `redirect_uris` and echoed to `/oauth/token`:
/// Mastodon requires all three to match exactly, so it is built once, here. The
/// router reads this constant, so the address bar's path and the registered URI
/// cannot drift apart.
pub(crate) const CALLBACK_PATH: &str = "/oauth/callback";

/// Starts the flow: register the client, stash the pending record, leave the page.
///
/// On success the browser is navigating to the instance, so the caller mostly sees
/// the error branch; `Ok(())` is returned right after `location.assign`.
///
/// # Errors
///
/// A message fit for an inline alert: bad host input, no usable page origin, the
/// instance refusing app registration, Web Crypto unavailable, or `sessionStorage`
/// blocked. Nothing is stored and no navigation happens when this returns `Err`.
pub async fn begin(host: &str) -> Result<(), String> {
    let host = normalise_host(host)?;
    let origin = origin()?;
    let redirect_uri = format!("{origin}{CALLBACK_PATH}");
    let client_id = api::register_app(&host, &origin, &redirect_uri).await?;

    let verifier = pkce::verifier()?;
    let challenge = pkce::challenge(&verifier).await?;
    let state = pkce::state()?;

    // Persist before redirecting: if this fails the flow cannot be completed, and the
    // user should hear it here rather than after a round trip to the instance.
    pending::save(&pending::Pending {
        host: host.clone(),
        client_id: client_id.clone(),
        verifier,
        state: state.clone(),
        redirect_uri: redirect_uri.clone(),
    })?;

    let url = authorize_url(&host, &client_id, &redirect_uri, &challenge, &state)?;
    window()?
        .location()
        .assign(&url)
        .map_err(|_| "Could not open the instance's authorization page.".to_owned())
}

/// Completes the flow from the callback's query string.
///
/// Returns `(host, access_token)`; the caller hands both to
/// [`crate::state::bridge::sign_in`], which verifies the token against the instance
/// before anything is installed or saved.
///
/// # Errors
///
/// A message fit for an inline alert: the instance refused (`error=access_denied`),
/// the callback is missing its code, there is no pending record in this tab, `state`
/// does not match, or the token exchange was rejected. The pending record is cleared
/// in every case, including these.
pub async fn finish(search: &str) -> Result<(String, String), String> {
    // Single use, and cleared before any early return: an authorization code must not
    // be replayable by a reload or a shared URL, and an abandoned attempt must not
    // leave a verifier sitting in storage.
    let record = pending::take();

    let params = web_sys::UrlSearchParams::new_with_str(search)
        .map_err(|_| "The callback URL could not be read.".to_owned())?;

    // Mastodon reports a refusal (the user pressed "Deny") as `error=access_denied`,
    // sometimes with a human-readable `error_description`.
    if let Some(error) = params.get("error") {
        let reason = params.get("error_description").unwrap_or(error);
        return Err(format!("The instance did not authorize the app: {reason}"));
    }

    let code = params
        .get("code")
        .filter(|code| !code.is_empty())
        .ok_or_else(|| "The instance's callback carried no authorization code.".to_owned())?;
    let state = params.get("state").unwrap_or_default();

    let record = record.ok_or_else(|| {
        "This sign-in did not start in this tab, so there is no pending request to compare \
         the instance's answer against. Start again from the sign-in page."
            .to_owned()
    })?;
    // The CSRF check: only an instance that saw our authorization request can echo the
    // `state` this tab generated.
    if state != record.state {
        return Err(
            "The callback's `state` does not match this tab's sign-in, so the answer \
                    was discarded. Start again from the sign-in page."
                .to_owned(),
        );
    }

    let token = api::exchange_code(&record, &code).await?;
    Ok((record.host, token))
}

/// The authorization URL, with `redirect_uri` percent-encoded by the `url` crate.
fn authorize_url(
    host: &str,
    client_id: &str,
    redirect_uri: &str,
    challenge: &str,
    state: &str,
) -> Result<String, String> {
    let mut url = Url::parse(&api::endpoint(host, "/oauth/authorize"))
        .map_err(|error| format!("Could not build the authorization URL: {error}"))?;
    url.query_pairs_mut()
        .append_pair("client_id", client_id)
        .append_pair("response_type", "code")
        .append_pair("redirect_uri", redirect_uri)
        .append_pair("scope", SCOPES)
        .append_pair("code_challenge", challenge)
        .append_pair("code_challenge_method", "S256")
        .append_pair("state", state);
    Ok(url.into())
}

/// Trims a pasted domain down to something that can be put into `https://{host}/…`.
///
/// This is a security boundary, not input polish: the value builds the URLs the code
/// and token travel to, so a scheme, a path, `@`, `?` or whitespace is refused rather
/// than interpreted.
fn normalise_host(input: &str) -> Result<String, String> {
    // A trailing slash is what the address bar gives when copied; it is the only
    // thing silently dropped. Everything else must already be a bare domain.
    let host = input.trim().trim_end_matches('/').to_ascii_lowercase();
    if host.is_empty() {
        return Err("Enter your instance's domain, for example mastodon.social.".to_owned());
    }
    if host.contains("://") {
        return Err("Enter only the domain, without the https:// prefix.".to_owned());
    }
    let charset_ok = host
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-'));
    // Empty labels catch a leading, trailing or doubled dot, which `Url::parse` would
    // otherwise accept and resolve somewhere unintended.
    let labels_ok = host.split('.').all(|label| !label.is_empty());
    if !charset_ok || !labels_ok {
        return Err(format!(
            "\"{host}\" is not an instance domain. Enter just the domain, for example \
             mastodon.social."
        ));
    }
    Ok(host)
}

/// This page's origin, which the instance needs as the redirect target.
///
/// `file://` pages report the literal string `null`, which cannot be registered and
/// would make the redirect fail after consent — so it is refused up front.
fn origin() -> Result<String, String> {
    let origin = window()?.location().origin().unwrap_or_default();
    if origin.starts_with("https://") || origin.starts_with("http://") {
        Ok(origin)
    } else {
        Err(
            "This page's origin cannot be used as a redirect target; serve the app over \
             http(s)."
                .to_owned(),
        )
    }
}

/// The browser window, or a message the user can act on.
fn window() -> Result<web_sys::Window, String> {
    web_sys::window().ok_or_else(|| "Sign-in needs a browser window.".to_owned())
}
