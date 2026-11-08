//! The SSR host: a cookie session in, HTML out.
//!
//! Scope of this slice, deliberately narrow: `/` and `/home` are painted by this process;
//! every other route and every asset still comes from the static bundle, which is the same
//! app. The page it paints carries the same bundle and a JSON island, so the moment the
//! wasm arrives it takes over with the data already on screen — no spinner, and no second
//! request for a page the server has already fetched.
//!
//! Configuration, all through the environment so a unit file can set it:
//!
//! - `NODUS_LISTEN` — `host:port` to bind (default `127.0.0.1:8080`).
//! - `NODUS_ASSET_DIR` — where `trunk` wrote the bundle (default `/srv/nodus`).
//! - `NODUS_SESSIONS` — the JSON file sessions survive a restart in (default
//!   `/var/lib/nodus/sessions.json`; set it to `-` for memory only).

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode, Uri, header};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;

mod mastodon;
mod html;
mod render;
mod session;

use mastodon::Client;
use render::Assets;
use session::{COOKIE, Session, Sessions};

/// How many statuses the server paints. One page, the same size the client asks for: the
/// point is to remove the first request, not to reimplement pagination.
const PAGE: u16 = 20;

#[derive(Debug)]
struct App {
    sessions: Arc<Sessions>,
    assets: std::sync::RwLock<Assets>,
    dist: PathBuf,
}

impl App {
    /// The asset names this process should serve, rediscovered when a deploy replaced them.
    fn bundle(&self) -> Assets {
        if let Ok(assets) = self.assets.read()
            && !assets.stale(&self.dist)
        {
            return assets.clone();
        }
        let fresh = Assets::discover(&self.dist);
        tracing::info!(script = %fresh.script, wasm = %fresh.wasm, "assets changed on disk");
        if let Ok(mut assets) = self.assets.write() {
            *assets = fresh.clone();
        }
        fresh
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,nodus_server=debug".into()),
        )
        .init();

    let listen = std::env::var("NODUS_LISTEN").unwrap_or_else(|_| "127.0.0.1:8080".to_owned());
    let asset_dir = std::env::var("NODUS_ASSET_DIR").unwrap_or_else(|_| "/srv/nodus".to_owned());
    let sessions_path = std::env::var("NODUS_SESSIONS")
        .unwrap_or_else(|_| "/var/lib/nodus/sessions.json".to_owned());

    let assets = Assets::discover(std::path::Path::new(&asset_dir));
    let sessions = Sessions::load((sessions_path != "-").then(|| PathBuf::from(&sessions_path)));
    tracing::info!(script = %assets.script, stylesheet = ?assets.stylesheet, "serving");

    let app = Arc::new(App {
        sessions: Arc::new(sessions),
        assets: std::sync::RwLock::new(assets),
        dist: PathBuf::from(&asset_dir),
    });

    let router = Router::new()
        .route("/healthz", get(healthz))
        .route("/api/session", get(whoami).post(sign_in).delete(sign_out))
        .route("/", get(index))
        .route("/home", get(home))
        .fallback(shell)
        .with_state(app);

    let address: SocketAddr = listen.parse()?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    tracing::info!(%address, "listening");
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown())
        .await?;
    Ok(())
}

async fn shutdown() {
    let _ = tokio::signal::ctrl_c().await;
    tracing::info!("stopping");
}

async fn healthz() -> impl IntoResponse {
    Json(json!({ "status": "ok" }))
}

/// The session cookie from a request, if it carries one.
fn cookie_of(headers: &HeaderMap) -> Option<String> {
    let raw = headers.get(header::COOKIE)?.to_str().ok()?;
    raw.split(';')
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(name, _)| *name == COOKIE)
        .map(|(_, value)| value.to_owned())
}

/// The `Set-Cookie` value. `HttpOnly` because no script needs it, `SameSite=Lax` so a
/// cross-site form cannot act as the reader, `Secure` because this host is TLS-only, and a
/// year's life because signing in again on every visit is the thing sessions exist to avoid.
fn cookie_for(id: &str) -> String {
    format!("{COOKIE}={id}; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age=31536000")
}

fn cleared_cookie() -> String {
    format!("{COOKIE}=; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age=0")
}

fn session_of(app: &App, headers: &HeaderMap) -> Option<Session> {
    app.sessions.get(&cookie_of(headers)?)
}

#[derive(Debug, Deserialize)]
struct Credentials {
    host: String,
    token: String,
}

/// Prove a token against its instance, then remember it behind an opaque id.
///
/// The token is proven before it is stored: a session that cannot read `verify_credentials`
/// would render an error page for every later request, and accepting it would make the
/// cookie a claim rather than a fact.
async fn sign_in(State(app): State<Arc<App>>, Json(body): Json<Credentials>) -> Response {
    let client = match Client::new(&body.host, &body.token) {
        Ok(client) => client,
        Err(message) => {
            return (StatusCode::BAD_REQUEST, Json(json!({ "error": message }))).into_response();
        }
    };
    match client.whoami().await {
        Ok(account) => {
            let id = app.sessions.insert(Session {
                host: body.host,
                token: body.token,
                account_id: account.id.clone(),
                acct: account.acct.clone(),
            });
            let mut response = Json(json!({
                "acct": account.acct,
                "account_id": account.id,
            }))
            .into_response();
            if let Ok(value) = cookie_for(&id).parse() {
                response.headers_mut().insert(header::SET_COOKIE, value);
            }
            response
        }
        Err(message) => {
            (StatusCode::UNAUTHORIZED, Json(json!({ "error": message }))).into_response()
        }
    }
}

async fn sign_out(State(app): State<Arc<App>>, headers: HeaderMap) -> Response {
    if let Some(id) = cookie_of(&headers) {
        app.sessions.remove(&id);
    }
    let mut response = StatusCode::NO_CONTENT.into_response();
    if let Ok(value) = cleared_cookie().parse() {
        response.headers_mut().insert(header::SET_COOKIE, value);
    }
    response
}

async fn whoami(State(app): State<Arc<App>>, headers: HeaderMap) -> Response {
    match session_of(&app, &headers) {
        Some(session) => Json(session.view()).into_response(),
        None => StatusCode::UNAUTHORIZED.into_response(),
    }
}

/// The front door. A signed-in visitor is sent to the painted home page; a stranger gets
/// the landing page, which is the same document minus the island.
async fn index(State(app): State<Arc<App>>, headers: HeaderMap) -> Response {
    if session_of(&app, &headers).is_some() {
        home(State(app), headers).await
    } else {
        Html(render::landing(&app.bundle())).into_response()
    }
}

/// Every route the client owns: the chrome, then the client takes over.
///
/// Assets never reach here (`Caddy` answers them from disk), so anything with a file
/// extension is a genuinely missing asset and must not become HTML.
async fn shell(State(app): State<Arc<App>>, headers: HeaderMap, uri: Uri) -> Response {
    let path = uri.path();
    if std::path::Path::new(path)
        .extension()
        .is_some_and(|_| !path.ends_with('/'))
    {
        return StatusCode::NOT_FOUND.into_response();
    }
    let acct = session_of(&app, &headers).map(|session| session.acct);
    Html(render::shell(&app.bundle(), acct.as_deref(), path)).into_response()
}

/// The home timeline, painted. This is the whole point of the slice: the request the
/// browser used to make after the wasm booted is made here, and its answer is the markup.
async fn home(State(app): State<Arc<App>>, headers: HeaderMap) -> Response {
    let Some(session) = session_of(&app, &headers) else {
        return Html(render::landing(&app.bundle())).into_response();
    };
    let client = match Client::new(&session.host, &session.token) {
        Ok(client) => client,
        Err(message) => return Html(render::error(&app.bundle(), &message)).into_response(),
    };
    match client.home(PAGE).await {
        Ok((statuses, cursor)) => Html(render::home(
            &app.bundle(),
            &session.acct,
            &statuses,
            cursor.as_deref(),
        ))
        .into_response(),
        Err(message) => Html(render::error(&app.bundle(), &message)).into_response(),
    }
}
