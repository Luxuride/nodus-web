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

mod html;
mod mastodon;
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

/// A public timeline, painted with the same body the home page uses.
async fn public_feed(State(app): State<Arc<App>>, headers: HeaderMap, local: bool) -> Response {
    let Some(session) = session_of(&app, &headers) else {
        return Html(render::landing(&app.bundle())).into_response();
    };
    let client = match Client::new(&session.host, &session.token) {
        Ok(client) => client,
        Err(message) => return Html(render::error(&app.bundle(), &message)).into_response(),
    };
    let (which, title, path) = if local {
        ("local", "Local", "/local")
    } else {
        ("federated", "Federated", "/federated")
    };
    match client.feed(local, PAGE).await {
        Ok((statuses, cursor)) => Html(render::feed(
            &app.bundle(),
            &session.acct,
            which,
            title,
            path,
            &statuses,
            cursor.as_deref(),
        ))
        .into_response(),
        Err(message) => Html(render::error(&app.bundle(), &message)).into_response(),
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
        .route(
            "/local",
            get(|state, headers| public_feed(state, headers, true)),
        )
        .route(
            "/federated",
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
                admin,
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
/// Where the reader holds a session, the page is painted with the data the client would
/// fetch anyway; every other path keeps the skeleton, which is also the fallback when a
/// read fails. `Caddy` answers assets from disk, so anything with a file extension reaching
/// here is genuinely missing.
async fn shell(State(app): State<Arc<App>>, headers: HeaderMap, uri: Uri) -> Response {
    let path = uri.path();
    if std::path::Path::new(path)
        .extension()
        .is_some_and(|_| !path.ends_with('/'))
    {
        return StatusCode::NOT_FOUND.into_response();
    }
    let Some(session) = session_of(&app, &headers) else {
        return Html(render::shell(&app.bundle(), None, path)).into_response();
    };
    // The rail draws the moderation doors for a moderator, and it is drawn before any
    // wasm runs, so the answer has to come from the session that was established at
    // sign-in (`Session::admin`).
    render::set_staff(session.admin);
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
        None => Html(render::shell(&assets, Some(&session.acct), path)).into_response(),
    }
}

/// The page for `path`, or `None` when this route has no server read.
///
/// A failed read is `None` as well: the skeleton is a worse page than the data, but a better
/// one than an error page for a screen the client can still fill in itself.
async fn read_page(
    client: &Client,
    assets: &render::Assets,
    acct: &str,
    path: &str,
    query: Option<&str>,
) -> Option<String> {
    let page = match path {
        "/notifications" => client.notifications(PAGE).await.map(|items| {
            render::list_page(
                assets,
                acct,
                path,
                "Notifications",
                "Notifications",
                "Follows, boosts, favourites, mentions and poll results, newest first.",
                &render::rows(
                    &items.iter().map(render::notification_row).collect::<Vec<_>>(),
                    "Nothing has happened yet.",
                ),
                Some(serde_json::json!({ "kind": "notifications", "items": items, "cursor": null }).to_string()),
            )
        }).ok(),
        "/messages" => client.conversations(PAGE).await.map(|items| {
            render::list_page(
                assets,
                acct,
                path,
                "Messages",
                "Messages",
                "Private conversations. A message is only ever visible to the people in it.",
                &render::rows(
                    &items.iter().map(render::conversation_row).collect::<Vec<_>>(),
                    "There are no conversations yet.",
                ),
                Some(serde_json::json!({ "kind": "conversations", "items": items, "cursor": null }).to_string()),
            )
        }).ok(),
        "/saved/favourites" | "/saved/bookmarks" => {
            read_saved(client, assets, acct, path, path.ends_with("bookmarks")).await
        }
        "/lists" => client.lists().await.map(|items| {
            render::list_page(
                assets,
                acct,
                path,
                "Lists",
                "Lists",
                "Curated groups of accounts, with their own timeline.",
                &render::rows(
                    &items.iter().map(render::list_row).collect::<Vec<_>>(),
                    "There are no lists yet.",
                ),
                Some(serde_json::json!({ "kind": "lists", "items": items, "cursor": null }).to_string()),
            )
        }).ok(),
        "/scheduled" => client.scheduled().await.map(|items| {
            render::list_page(
                assets,
                acct,
                path,
                "Scheduled",
                "Scheduled",
                "Posts waiting to be published. Write one from the box above the home timeline.",
                &render::rows(
                    &items.iter().map(render::scheduled_row).collect::<Vec<_>>(),
                    "Nothing is scheduled.",
                ),
                Some(serde_json::json!({ "kind": "scheduled", "items": items, "cursor": null }).to_string()),
            )
        }).ok(),
        "/directory" => client.directory(true).await.map(|items| {
            render::list_page(
                assets,
                acct,
                path,
                "People",
                "People",
                "Accounts this instance suggests, ordered by how recently they were active.",
                &render::rows(
                    &items.iter().map(render::account_row).collect::<Vec<_>>(),
                    "This instance suggests nobody yet.",
                ),
                Some(serde_json::json!({ "kind": "directory", "items": items, "cursor": null }).to_string()),
            )
        }).ok(),
        "/search" => read_search(client, assets, acct, path, query).await,
        _ if path.starts_with("/search") => None,
        _ => None,
    };
    if page.is_some() {
        return page;
    }
    read_entity(client, assets, acct, path).await
}

/// Before the routes with a name in them: one account, or one thread.
async fn read_entity(
    client: &Client,
    assets: &render::Assets,
    acct: &str,
    path: &str,
) -> Option<String> {
    if let Some(id) = path.strip_prefix("/accounts/") {
        if !id.is_empty() && !id.contains('/') {
            let account = client.account(id).await.ok()?;
            let (statuses, cursor) = client.account_statuses(id, PAGE).await.ok()?;
            return Some(render::profile_page(
                assets,
                acct,
                &account,
                &statuses,
                cursor.as_deref(),
            ));
        }
        return None;
    }
    let id = path.strip_prefix("/status/")?;
    if id.is_empty() || id.contains('/') {
        return None;
    }
    let status = client.status(id).await.ok()?;
    let context = client.context(id).await.ok()?;
    Some(render::thread_page(assets, acct, &status, &context))
}

/// Favourites or bookmarks: one screen, two shelves, the same shape.
async fn read_saved(
    client: &Client,
    assets: &render::Assets,
    acct: &str,
    path: &str,
    bookmarks: bool,
) -> Option<String> {
    let (statuses, cursor) = client.saved(bookmarks, PAGE).await.ok()?;
    let (title, description) = if bookmarks {
        (
            "Bookmarks",
            "Posts you kept to come back to. Only you can see this list.",
        )
    } else {
        (
            "Favourites",
            "Posts you starred. Only you can see this list.",
        )
    };
    Some(render::list_page(
        assets,
        acct,
        path,
        title,
        title,
        description,
        &render::rows(
            &statuses.iter().map(render::status_card).collect::<Vec<_>>(),
            "Nothing saved yet.",
        ),
        Some(
            serde_json::json!({
                "kind": if bookmarks { "bookmarks" } else { "favourites" },
                "items": statuses,
                "cursor": cursor,
            })
            .to_string(),
        ),
    ))
}

/// The search page, the one route whose argument comes from the query string.
async fn read_search(
    client: &Client,
    assets: &render::Assets,
    acct: &str,
    path: &str,
    query: Option<&str>,
) -> Option<String> {
    let query = query
        .and_then(|query| {
            query.split('&').find_map(|pair| {
                let (name, value) = pair.split_once('=')?;
                (name == "q").then(|| percent_decode(value))
            })
        })
        .unwrap_or_default();
    if query.trim().is_empty() {
        return None;
    }
    let results = client.search(&query).await.ok()?;
    let mut rows = results
        .accounts
        .iter()
        .map(render::account_row)
        .collect::<Vec<_>>();
    rows.extend(results.statuses.iter().map(render::status_card));
    Some(render::list_page(
        assets,
        acct,
        path,
        "Search",
        "Search",
        &format!("Results for \"{query}\""),
        &render::rows(&rows, "Nothing matched."),
        None,
    ))
}

/// Percent-decoding for the one query parameter this host reads. `+` is a space because a
/// search box is what sends it.
fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'+' => {
                decoded.push(b' ');
                index += 1;
            }
            b'%' if index + 2 < bytes.len() => {
                let hex = std::str::from_utf8(&bytes[index + 1..index + 3]).ok();
                if let Some(byte) = hex.and_then(|hex| u8::from_str_radix(hex, 16).ok()) {
                    decoded.push(byte);
                    index += 3;
                } else {
                    decoded.push(bytes[index]);
                    index += 1;
                }
            }
            byte => {
                decoded.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8_lossy(&decoded).into_owned()
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
