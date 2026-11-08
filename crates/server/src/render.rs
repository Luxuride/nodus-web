//! The HTML a reader sees before any wasm runs.
//!
//! This is a *server-painted* page, not a hydrated Leptos tree: the same `index.html`
//! contract (one stylesheet, one module script, `data-theme`), the statuses as real markup,
//! and a JSON island the client adopts so it does not re-fetch what the server already
//! asked for. Hydration proper can come later; what matters first is that the first paint
//! is content rather than a spinner.
//!
//! Two rules hold the design together, both from `nodus-backend/docs/06-ssr.md`:
//!
//! - **No token in the output.** Not in the markup, not in the island. The island carries
//!   data and a cursor.
//! - **Unknown is not a default.** When the server has no session it paints the signed-out
//!   page; it never paints one account's page and lets the client swap it.

use nodus_instance::model::Status;

/// The assets `trunk` produced. Discovered at startup, because the names carry content
/// hashes: writing them down here would mean a stale page after every build.
#[derive(Debug, Clone)]
pub struct Assets {
    pub script: String,
    pub stylesheet: Option<String>,
}

impl Assets {
    /// Find the bundle in a `dist` directory. Falls back to trunk's stable names so a
    /// misconfigured path produces a page that still works rather than a 404 loop.
    #[must_use]
    pub fn discover(dist: &std::path::Path) -> Self {
        let mut script = None;
        let mut stylesheet = None;
        if let Ok(entries) = std::fs::read_dir(dist) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                let path = std::path::Path::new(&name);
                let is_js = path
                    .extension()
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("js"));
                let is_css = path
                    .extension()
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("css"));
                if name.starts_with("nodus-web") && is_js {
                    script = Some(format!("/{name}"));
                } else if name.starts_with("tailwind") && is_css {
                    stylesheet = Some(format!("/{name}"));
                }
            }
        }
        Self {
            script: script.unwrap_or_else(|| "/nodus-web.js".to_owned()),
            stylesheet,
        }
    }
}

/// Escape text for HTML. The status body is HTML from the instance, which this server
/// cannot sanitise honestly — so it renders as text, and the client (which has a parser and
/// a sanitiser) is what turns it back into a formatted body after it takes over.
#[must_use]
pub fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            other => out.push(other),
        }
    }
    out
}

/// The island: what the client should adopt rather than fetch.
#[derive(Debug, serde::Serialize)]
pub struct Island<'a> {
    pub which: &'static str,
    pub statuses: &'a [Status],
    pub cursor: Option<&'a str>,
}

/// One status as markup: who, when, what.
fn status_card(status: &Status) -> String {
    let boosted = status
        .reblog
        .as_ref()
        .map(|_| {
            format!(
                "<p class=\"text-xs opacity-60\">{} boosted</p>",
                escape(&status.account.acct)
            )
        })
        .unwrap_or_default();
    let inner: &Status = status.reblog.as_deref().unwrap_or(status);
    let handle = format!("@{}{}", escape(&inner.account.acct), "");
    let mut media = String::new();
    for url in inner
        .media_attachments
        .iter()
        .filter_map(|attachment| attachment.url.as_deref())
    {
        let _ = std::fmt::Write::write_fmt(
            &mut media,
            format_args!(
                "<a class=\"mt-2 block text-xs underline\" href=\"{}\">attachment</a>",
                escape(url)
            ),
        );
    }
    format!(
        "<li class=\"card border border-base-300 bg-base-100 shadow-sm\">\
           <div class=\"card-body gap-1 p-4\">{boosted}\
             <p class=\"text-sm font-semibold\">{name} <span class=\"font-normal opacity-60\">{handle}</span></p>\
             <p class=\"text-xs opacity-60\">{when}</p>\
             <div class=\"text-sm whitespace-pre-wrap\">{body}</div>{media}\
           </div></li>",
        name = escape(&inner.account.display_name),
        when = escape(&inner.created_at),
        body = escape(&plain(&inner.content)),
    )
}

/// The body with its tags removed: an approximation of what the client will render, and
/// honest about being an approximation — the island carries the HTML for the real thing.
fn plain(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut inside = false;
    for character in html.chars() {
        match character {
            '<' => inside = true,
            '>' => inside = false,
            other if !inside => out.push(other),
            _ => {}
        }
    }
    out.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

/// The signed-in home page, painted.
pub fn home(assets: &Assets, acct: &str, statuses: &[Status], cursor: Option<&str>) -> String {
    let cards = statuses.iter().map(status_card).collect::<String>();
    let island = serde_json::to_string(&Island {
        which: "home",
        statuses,
        cursor,
    })
    .unwrap_or_else(|_| "{\"which\":\"home\",\"statuses\":[],\"cursor\":null}".to_owned());
    let more = cursor
        .map(|cursor| {
            format!(
                "<button class=\"btn btn-ghost btn-sm w-full\" data-cursor=\"{}\">Load more</button>",
                escape(cursor)
            )
        })
        .unwrap_or_default();
    page(
        assets,
        &format!("Home · {acct}"),
        &format!(
            "<section class=\"mx-auto flex w-full max-w-2xl flex-col gap-3\">\
               <h1 class=\"text-xl font-semibold tracking-tight\">Home</h1>\
               <ul class=\"flex flex-col gap-2\">{cards}</ul>{more}\
             </section>"
        ),
        Some(&island),
    )
}

/// The signed-out page: the app's own front door, painted without data.
pub fn landing(assets: &Assets) -> String {
    page(
        assets,
        "Nodus",
        "<section class=\"mx-auto flex w-full max-w-2xl flex-col gap-3\">\
           <h1 class=\"text-xl font-semibold tracking-tight\">Nodus</h1>\
           <p class=\"text-sm opacity-70\">A Mastodon client that keeps its state, so a page you have \
             already read does not load again.</p>\
           <p class=\"text-sm opacity-70\">Sign in to see your timelines.</p>\
         </section>",
        None,
    )
}

/// A page that says what went wrong and still loads the app, so the reader can sign out or
/// retry instead of staring at an empty document.
pub fn error(assets: &Assets, message: &str) -> String {
    page(
        assets,
        "Timeline unavailable",
        &format!(
            "<section class=\"mx-auto w-full max-w-2xl\">\
               <h1 class=\"text-xl font-semibold tracking-tight\">Home</h1>\
               <p class=\"alert alert-error mt-2 text-sm\" role=\"alert\">Could not read the timeline: {}</p>\
             </section>",
            escape(message)
        ),
        None,
    )
}

/// The shell both pages share: the app's stylesheet, its bundle, and the island.
fn page(assets: &Assets, title: &str, body: &str, island: Option<&str>) -> String {
    let stylesheet = assets
        .stylesheet
        .as_ref()
        .map(|href| format!("<link rel=\"stylesheet\" href=\"{}\">", escape(href)))
        .unwrap_or_default();
    let island = island
        .map(|json| {
            format!(
                "<script id=\"nodus-page\" type=\"application/json\">{}</script>",
                json.replace("</", "<\\/")
            )
        })
        .unwrap_or_default();
    format!(
        "<!doctype html>\
         <html lang=\"en\" data-theme=\"light\">\
           <head>\
             <meta charset=\"utf-8\">\
             <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
             <title>{title}</title>\
             <script>(function(){{var d=window.matchMedia&&window.matchMedia('(prefers-color-scheme: dark)').matches;document.documentElement.dataset.theme=d?'dark':'light';}})()</script>\
             {stylesheet}\
             <script type=\"module\" src=\"{script}\"></script>\
           </head>\
           <body class=\"min-h-screen bg-base-200\">\
             <main id=\"app\" class=\"px-2 py-4\">{body}</main>\
             {island}\
           </body>\
         </html>",
        title = escape(title),
        script = escape(&assets.script),
    )
}
