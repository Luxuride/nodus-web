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

use std::fmt::Write as _;

use nodus_instance::model::{MediaKind, Status};

/// The assets `trunk` produced. Discovered at startup, because the names carry content
/// hashes: writing them down here would mean a stale page after every build.
#[derive(Debug, Clone)]
pub struct Assets {
    pub script: String,
    pub wasm: String,
    pub stylesheet: Option<String>,
}

impl Assets {
    /// Find the bundle in a `dist` directory. Falls back to trunk's stable names so a
    /// misconfigured path produces a page that still works rather than a 404 loop.
    #[must_use]
    pub fn discover(dist: &std::path::Path) -> Self {
        let mut script = None;
        let mut wasm = None;
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
                let is_wasm = path
                    .extension()
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("wasm"));
                if name.starts_with("nodus-web") && is_js {
                    script = Some(format!("/{name}"));
                } else if name.starts_with("nodus-web") && is_wasm {
                    wasm = Some(format!("/{name}"));
                } else if name.starts_with("tailwind") && is_css {
                    stylesheet = Some(format!("/{name}"));
                }
            }
        }
        Self {
            script: script.unwrap_or_else(|| "/nodus-web.js".to_owned()),
            wasm: wasm.unwrap_or_else(|| "/nodus-web_bg.wasm".to_owned()),
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

/// Turns a status body into text that keeps the author's line breaks.
///
/// Mastodon answers with HTML fragments (`<p>…</p>`, `<br>`, links, `<span>` for
/// mentions). Escaping that markup shows the tags themselves, and collapsing it loses
/// every newline, so the tags are turned into the whitespace they stand for and the rest
/// is stripped — the rendered page is *text*, and the client's own renderer takes over
/// the moment wasm boots.
fn body_text(html: &str) -> String {
    let mut text = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(index) = rest.find('<') {
        text.push_str(&rest[..index]);
        let Some(end) = rest[index..].find('>') else {
            rest = &rest[index..];
            break;
        };
        let tag = &rest[index..=index + end];
        let lower = tag.to_ascii_lowercase();
        // Every tag that stands for a break becomes the whitespace it means; a paragraph
        // opening *and* closing both cost a newline is acceptable, and collapsing them is
        // what produced one long run-on paragraph before.
        if lower.starts_with("<br") || lower.starts_with("<p") || lower.starts_with("</p") || lower.starts_with("</div") {
            text.push('\n');
        }
        rest = &rest[index + end + 1..];
    }
    text.push_str(rest);
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .trim()
        .to_owned()
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
    for attachment in &inner.media_attachments {
        let Some(url) = attachment.url.as_deref() else {
            continue;
        };
        let href = escape(url);
        let poster = attachment
            .preview_url
            .as_deref()
            .map(|preview| format!(" poster=\"{}\"", escape(preview)))
            .unwrap_or_default();
        let alt = attachment
            .description
            .as_deref()
            .map(|description| format!(" alt=\"{}\"", escape(description)))
            .unwrap_or_default();
        let element = match attachment.kind {
            // A GIF in Mastodon is a muted video, so it plays in place and loops.
            MediaKind::Video | MediaKind::Gifv => format!(
                "<video class=\"mt-2 w-full rounded\" controls playsinline preload=\"metadata\" loop{poster} src=\"{href}\"></video>"
            ),
            MediaKind::Audio => format!(
                "<audio class=\"mt-2 w-full\" controls preload=\"metadata\" src=\"{href}\"></audio>"
            ),
            MediaKind::Image | MediaKind::Unknown => format!(
                "<a class=\"mt-2 block\" href=\"{href}\"><img class=\"w-full rounded\" loading=\"lazy\"{alt} src=\"{href}\"></a>"
            ),
        };
        let _ = std::fmt::Write::write_str(&mut media, &element);
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
        body = escape(&body_text(&inner.content)),
    )
}


/// The destinations the rail offers, in the order the client renders them.
type Entry = (&'static str, &'static str, &'static str);
type Section = (&'static str, [Entry; 3]);

const RAIL: [Section; 3] = [
    (
        "Read",
        [
            ("Home", "/home", "Your timeline, the feed, and the instance's"),
            ("Messages", "/messages", "Direct conversations"),
            (
                "Notifications",
                "/notifications",
                "Mentions, boosts, favourites and follow requests",
            ),
        ],
    ),
    (
        "Find",
        [
            (
                "Search",
                "/search",
                "Accounts, posts and hashtags, including remote ones",
            ),
            ("People", "/directory", "Accounts this instance suggests"),
            (
                "Saved",
                "/saved/favourites",
                "Your favourites and bookmarks",
            ),
        ],
    ),
    (
        "Library",
        [
            ("Scheduled", "/scheduled", "Posts waiting to be published"),
            (
                "Lists",
                "/lists",
                "Hand-picked accounts with a timeline each",
            ),
            (
                "Settings",
                "/settings/account",
                "Profile, tags, filters, sync and this instance",
            ),
        ],
    ),
];

/// The rail, painted with the same classes the client uses.
///
/// The icons are squares of the icon's own size rather than glyphs: the client swaps in
/// the real paths on mount, and matching the *box* is what keeps the column from moving.
/// Copying twenty-seven SVGs into this server would be a second icon set to keep in sync
/// for a difference nobody sees after the first paint.
fn nav() -> String {
    let mut items = String::new();
    for (section, entries) in RAIL {
        let _ = write!(
            items,
            "<li class=\"menu-title hidden xl:block\">{}</li>",
            escape(section)
        );
        for (label, href, hint) in entries {
            let _ = write!(
                items,
                "<li>\
                   <a class=\"flex w-full items-center gap-2 font-medium\" href=\"{}\" title=\"{}\">\
                     <span class=\"inline-block h-4 w-4 shrink-0 rounded-sm bg-base-300\"></span>\
                     <span class=\"truncate\">{}</span>\
                   </a>\
                 </li>",
                escape(href),
                escape(hint),
                escape(label)
            );
        }
    }
    format!(
        "<nav class=\"hidden shrink-0 xl:block xl:w-44\">\
           <ul class=\"menu w-full flex-row flex-wrap gap-1 rounded-box bg-base-100 p-2 shadow-sm xl:flex-col xl:flex-nowrap\">{items}</ul>\
         </nav>"
    )
}

/// The application's own frame: the same root element, column widths and gaps the client
/// renders, so the swap is a replacement in place rather than a reflow.
fn chrome(signed_in: bool, inner: &str) -> String {
    if !signed_in {
        return format!(
            "<div class=\"min-h-screen bg-base-200 text-base-content\">\
               <div class=\"mx-auto flex w-full max-w-6xl flex-col gap-4 px-2 py-4\">\
                 <main class=\"min-w-0 flex-1\">{inner}</main>\
               </div>\
             </div>"
        );
    }
    format!(
        "<div class=\"min-h-screen bg-base-200 text-base-content\">\
           <header class=\"sticky top-0 z-30 flex items-center justify-between gap-2 border-b border-base-300 bg-base-100 px-3 py-2 xl:hidden\">\
             <a class=\"font-semibold tracking-tight\" href=\"/home\">Nodus</a>\
           </header>\
           <div class=\"mx-auto flex w-full max-w-6xl flex-col gap-4 px-2 py-4 xl:flex-row\">{}\n\
             <main class=\"min-w-0 flex-1 pb-[calc(5.5rem+env(safe-area-inset-bottom))] xl:pb-0\">{inner}</main>\
           </div>\
         </div>",
        nav()
    )
}

/// The composer and the timeline tabs, at the size the client gives them.
fn timeline_header() -> String {
    "<div class=\"sticky top-0 z-20 -mx-1 flex flex-col gap-2 bg-base-200/95 px-1 pb-2 pt-1 backdrop-blur\">\
       <div class=\"card border border-base-300 bg-base-100 shadow-sm\">\
         <div class=\"card-body gap-2 p-3\">\
           <textarea class=\"textarea textarea-ghost h-16 w-full resize-none text-base\" aria-label=\"Write a post\" placeholder=\"What is on your mind?\"></textarea>\
           <div class=\"flex items-center gap-2\">\
             <span class=\"btn btn-outline btn-sm gap-1 border-dashed\">+</span>\
             <span class=\"select select-bordered select-sm\">Public</span>\
             <span class=\"ml-auto text-xs opacity-50\">0/500</span>\
             <span class=\"btn btn-sm btn-primary\">Post</span>\
           </div>\
         </div>\
       </div>\
       <div role=\"tablist\" class=\"tabs tabs-box\">\
         <a role=\"tab\" class=\"tab tab-active\" href=\"/home\">Home</a>\
         <a role=\"tab\" class=\"tab\" href=\"/feed\">Feed</a>\
         <a role=\"tab\" class=\"tab\" href=\"/local\">Local</a>\
         <a role=\"tab\" class=\"tab\" href=\"/federated\">Federated</a>\
       </div>\
     </div>"
        .to_owned()
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
        &chrome(
            true,
            &format!(
                "<div class=\"mx-auto flex w-full max-w-2xl flex-col gap-3\">{header}\
                   <ul class=\"flex flex-col gap-2\">{cards}</ul>{more}\
                 </div>",
                header = timeline_header()
            ),
        ),
        Some(&island),
    )
}

/// The signed-out page: the app's own front door, painted without data.
pub fn landing(assets: &Assets) -> String {
    page(
        assets,
        "Nodus",
        &chrome(
            false,
            "<div class=\"mx-auto flex w-full max-w-2xl flex-col gap-3\">\
               <h1 class=\"text-xl font-semibold tracking-tight\">Nodus</h1>\
               <p class=\"text-sm opacity-70\">A Mastodon client that keeps its state, so a page you \
                 have already read does not load again.</p>\
               <p class=\"text-sm opacity-70\">Sign in to see your timelines.</p>\
             </div>",
        ),
        None,
    )
}

/// The shell for every route the client owns.
///
/// The home timeline cannot be painted for a screen the server does not implement (a
/// thread, a profile, the settings pages), so those routes get the chrome: the wordmark,
/// the navigation, and the note that says what is being opened. The client replaces the
/// body the moment it boots, and its store answers the screen from cache when it can.
/// Painting the chrome is still worth it — the reader sees the application, not a blank
/// document, while two megabytes of wasm arrive.
pub fn shell(assets: &Assets, acct: Option<&str>, path: &str) -> String {
    let who = acct.map_or_else(
        || "<p class=\"text-sm opacity-70\">Sign in to see your timelines.</p>".to_owned(),
        |acct| format!("<p class=\"text-sm opacity-70\">Signed in as {}</p>", escape(acct)),
    );
    page(
        assets,
        "Nodus",
        &chrome(
            acct.is_some(),
            &format!(
                "<div class=\"mx-auto flex w-full max-w-2xl flex-col gap-2\">\
                   <h1 class=\"text-xl font-semibold tracking-tight\">Nodus</h1>{who}\
                   <p class=\"text-xs opacity-50\">Opening {}</p>\
                 </div>",
                escape(path)
            ),
        ),
        None,
    )
}

/// A page that says what went wrong and still loads the app, so the reader can sign out or
/// retry instead of staring at an empty document.
pub fn error(assets: &Assets, message: &str) -> String {
    page(
        assets,
        "Timeline unavailable",
        &chrome(
            true,
            &format!(
                "<div class=\"mx-auto w-full max-w-2xl\">\
                   <h1 class=\"text-xl font-semibold tracking-tight\">Home</h1>\
                   <p class=\"alert alert-error mt-2 text-sm\" role=\"alert\">Could not read the timeline: {}</p>\
                 </div>",
                escape(message)
            ),
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
             <script type=\"module\">\
               import init, * as bindings from '{script}';\
               await init({{ module_or_path: '{wasm}' }});\
               window.wasmBindings = bindings;\
             </script>\
           </head>\
           <body class=\"min-h-screen bg-base-200\">\
             <div id=\"app\">{body}</div>\
             {island}\
           </body>\
         </html>",
        title = escape(title),
        script = escape(&assets.script),
        wasm = escape(&assets.wasm),
    )
}
