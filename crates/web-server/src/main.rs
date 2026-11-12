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

mod content;
mod document;
mod html;
mod mastodon;
mod pages;
mod rows;
mod runtime;
mod session;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode, Uri, header};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::get;
use axum::{Json, Router};
use nodus_web::shared::router::Route;
use serde::Deserialize;
use serde_json::json;

use crate::document::Assets;
use crate::mastodon::Client;
use crate::pages::read_page;
use crate::session::{COOKIE, Session, Sessions};

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

/// A public timeline, painted with the same body the home page uses.
async fn public_feed(State(app): State<Arc<App>>, headers: HeaderMap, local: bool) -> Response {
    let Some(session) = session_of(&app, &headers) else {
        return Html(document::landing(&app.bundle())).into_response();
    };
    let client = match Client::new(&session.host, &session.token) {
        Ok(client) => client,
        Err(message) => return Html(document::error(&app.bundle(), &message)).into_response(),
    };
    let (which, title, path) = if local {
        ("local", "Local", "/home/local")
    } else {
        ("federated", "Federated", "/home/federated")
    };
    match client.feed(local, PAGE).await {
        Ok((statuses, cursor)) => Html(document::feed(
            &app.bundle(),
            &session.acct,
            which,
            title,
            path,
            &statuses,
            cursor.as_deref(),
        ))
        .into_response(),
        Err(message) => Html(document::error(&app.bundle(), &message)).into_response(),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,nodus_web=debug".into()),
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
        .route(
            "/home/local",
            get(|state, headers| public_feed(state, headers, true)),
        )
        .route(
            "/home/federated",
            get(|state, headers| public_feed(state, headers, false)),
        )
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
            let admin = client.is_staff().await;
            let id = app.sessions.insert(Session {
                host: body.host,
                token: body.token,
                account_id: account.id.clone(),
                acct: account.acct.clone(),
                admin: Some(admin),
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
        Redirect::to("/home").into_response()
    } else {
        Html(document::landing(&app.bundle())).into_response()
    }
}

/// Every route the client owns: the chrome, then the client takes over.
///
/// Where the reader holds a session, the page is painted with the data the client would
/// fetch anyway; every other path keeps the skeleton, which is also the fallback when a
/// read fails. `Caddy` answers assets from disk, so anything with a file extension reaching
/// here is genuinely missing.
/// A shared link without a cookie: read the post and its thread from its own host.
async fn public_thread(app: &App, path: &str) -> Option<String> {
    let Route::Thread { host, id } = Route::from_path(path) else {
        return None;
    };
    if host.is_empty() {
        return None;
    }
    let client = match Client::new(&host, "") {
        Ok(client) => client,
        Err(message) => {
            tracing::warn!(%message, %host, "public post host could not be used");
            return None;
        }
    };
    let status = match client.status(&id).await {
        Ok(status) => status,
        Err(message) => {
            tracing::warn!(%message, %host, %id, "public post could not be read");
            return None;
        }
    };
    let context = client
        .context(&id)
        .await
        .unwrap_or(nodus_instance::model::Context {
            ancestors: Vec::new(),
            descendants: Vec::new(),
        });
    Some(pages::render_public_thread(
        &app.bundle(),
        &host,
        &id,
        status,
        context,
    ))
}

/// An account linked from a public post: read from its own host, without a token.
async fn public_account(app: &App, path: &str) -> Option<String> {
    let Route::Account(rest) = Route::from_path(path) else {
        return None;
    };
    let (host, id) = rest.split_once('/')?;
    if host.is_empty() || id.is_empty() {
        return None;
    }
    let client = Client::new(host, "").ok()?;
    let account = client.account(id).await.ok()?;
    let (statuses, _) = client.account_statuses(id, PAGE).await.ok()?;
    Some(pages::render_public_account(
        &app.bundle(),
        host,
        id,
        account,
        statuses,
    ))
}

async fn shell(State(app): State<Arc<App>>, headers: HeaderMap, uri: Uri) -> Response {
    let path = uri.path();
    if std::path::Path::new(path)
        .extension()
        .is_some_and(|_| !path.ends_with('/'))
    {
        return StatusCode::NOT_FOUND.into_response();
    }
    let Some(session) = session_of(&app, &headers) else {
        if let Some(id) = path.strip_prefix("/accounts/") {
            if !id.contains('/') {
                if let Ok(host) = std::env::var("NODUS_INSTANCE_HOST") {
                    if !host.trim().is_empty() {
                        return Redirect::to(&format!("/accounts/{}/{id}", host.trim()))
                            .into_response();
                    }
                }
            }
        }
        if let Some(html) = public_thread(&app, path).await {
            return Html(html).into_response();
        }
        if let Some(html) = public_account(&app, path).await {
            return Html(html).into_response();
        }
        return Html(document::shell(&app.bundle(), None, path)).into_response();
    };
    // The rail draws the moderation doors for a moderator, and it is drawn before any
    // wasm runs, so the answer has to come from the session that was established at
    // sign-in (`Session::admin`).
    let admin = if let Some(admin) = session.admin {
        admin
    } else {
        {
            let answer = match Client::new(&session.host, &session.token) {
                Ok(client) => client.is_staff().await,
                Err(message) => {
                    tracing::warn!(%message, "could not ask whether this account moderates");
                    false
                }
            };
            app.sessions.set_admin(&session.host, &session.acct, answer);
            answer
        }
    };
    document::set_staff(admin);
    let assets = app.bundle();
    let painted = match Client::new(&session.host, &session.token) {
        Ok(client) => read_page(&client, &assets, &session.acct, path, uri.query()).await,
        Err(message) => {
            tracing::warn!(%message, "session could not be used; painting the skeleton");
            None
        }
    };
    match painted {
        Some(html) => Html(html).into_response(),
        None => Html(document::shell(&assets, Some(&session.acct), path)).into_response(),
    }
}

/// The page for `path`, or `None` when this route has no server read.
///
/// A failed read is `None` as well: the skeleton is a worse page than the data, but a better
/// one than an error page for a screen the client can still fill in itself.
async fn home(State(app): State<Arc<App>>, headers: HeaderMap) -> Response {
    let Some(session) = session_of(&app, &headers) else {
        return Html(document::landing(&app.bundle())).into_response();
    };
    let client = match Client::new(&session.host, &session.token) {
        Ok(client) => client,
        Err(message) => return Html(document::error(&app.bundle(), &message)).into_response(),
    };
    match client.home(PAGE).await {
        Ok((statuses, cursor)) => Html(document::home(
            &app.bundle(),
            &session.acct,
            &statuses,
            cursor.as_deref(),
        ))
        .into_response(),
        Err(message) => Html(document::error(&app.bundle(), &message)).into_response(),
    }
}
