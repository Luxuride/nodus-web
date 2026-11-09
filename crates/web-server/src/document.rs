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

use leptos::prelude::Owner;
use leptos::tachys::html::element::InnerHtmlAttribute as _;
use leptos::tachys::view::RenderHtml as _;
use leptos::tachys::view::any_view::IntoAny as _;

use nodus_instance::model::{Context, Status};

// The rows live beside the page builders: they are the parts a page is made of, and they are
// the only place markup is produced for them.
pub(crate) use crate::rows::*;

/// The assets `trunk` produced. Discovered at startup, because the names carry content
/// hashes: writing them down here would mean a stale page after every build.
#[derive(Debug, Clone)]
pub struct Assets {
    pub script: String,
    pub wasm: String,
    /// The stylesheet's path, kept so a deploy that changes only the CSS is visible.
    pub stylesheet: Option<String>,
    /// The stylesheet's contents, inlined into every page. One request fewer before
    /// the first paint, and no second asset whose hash can go stale during a deploy.
    pub stylesheet_css: Option<String>,
}

impl Assets {
    /// Whether the files this build discovered are still the ones on disk.
    ///
    /// `Caddy` serves `/srv/nodus` from the filesystem, so a static deploy replaces both the
    /// script and the wasm while this process keeps serving. Without this check the page
    /// keeps naming the previous bundle, the browser asks for a file that no longer exists,
    /// and the wasm never boots — the document stays server-painted and looks like SSR
    /// having replaced CSR.
    #[must_use]
    pub fn stale(&self, dist: &std::path::Path) -> bool {
        if !dist.join(&self.wasm).is_file() || !dist.join(&self.script).is_file() {
            return true;
        }
        self.stylesheet
            .as_ref()
            .is_some_and(|path| !dist.join(path).is_file())
    }

    /// Find the bundle in a `dist` directory. Falls back to trunk's stable names so a
    /// misconfigured path produces a page that still works rather than a 404 loop.
    pub fn discover(dist: &std::path::Path) -> Self {
        let mut script = None;
        let mut wasm = None;
        let mut stylesheet = None;
        let mut stylesheet_css = None;
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
                    stylesheet_css = std::fs::read_to_string(entry.path()).ok();
                }
            }
        }
        Self {
            script: script.unwrap_or_else(|| "/nodus-web.js".to_owned()),
            wasm: wasm.unwrap_or_else(|| "/nodus-web_bg.wasm".to_owned()),
            stylesheet,
            stylesheet_css,
        }
    }
}

/// Escapes text for HTML attributes and text nodes.
///
/// Status bodies are not escaped here any more: they are rewritten by [`crate::html`],
/// which keeps the parts of Mastodon's markup that a post is made of.
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
    pub which: &'a str,
    pub statuses: &'a [Status],
    pub cursor: Option<&'a str>,
}

// Whether this request's reader may moderate. Set once per request by `shell`; a
// thread-local rather than a parameter because every page function would otherwise have to
// thread a flag it does not use, and the server handles one request per thread.
thread_local! {
    static STAFF: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Whether this request's reader may moderate.
pub fn set_staff(staff: bool) {
    STAFF.with(|state| state.set(staff));
}

/// The two doors a moderator has, drawn from the same list the client draws.
fn staff_doors() -> leptos::prelude::AnyView {
    use imperative::rail_rows;
    rail_rows()
}

/// Markup for the extra rail rows, kept beside the frame so both come from one place.
mod imperative {
    use leptos::prelude::*;

    /// The moderation and administration rows.
    #[must_use]
    pub fn rail_rows() -> AnyView {
        view! {
            <nodus_view::RailEntry
                label="Moderation"
                href="/moderation"
                hint="The queue, the accounts and what the server allows"
                icon=nodus_view::icons::Icon::Shield
                active=Signal::derive(|| false)
            />
            <nodus_view::RailEntry
                label="Administration"
                href="/admin"
                hint="What this instance publishes and allows"
                icon=nodus_view::icons::Icon::Globe
                active=Signal::derive(|| false)
            />
        }
        .into_any()
    }
}

fn chrome(signed_in: bool, path: &str, inner: &str) -> String {
    use leptos::prelude::{IntoAny, Owner, Signal};

    // The frame, the rail and the icons come from `nodus-view`, the same components the
    // browser mounts. That is the whole point: two templates drift, one tree cannot.
    let path = path.to_owned();
    let inner = inner.to_owned();
    Owner::new().with(|| {
        let body = inner.clone();
        nodus_view::Frame(nodus_view::FrameProps {
            signed_in: Signal::derive(move || signed_in),
            path: Signal::derive({
                let path = path.clone();
                move || path.clone()
            }),
            inner: std::sync::Arc::new(move || {
                leptos::html::div().inner_html(body.clone()).into_any()
            }),
            extra: STAFF
                .with(std::cell::Cell::get)
                .then(|| std::sync::Arc::new(staff_doors) as nodus_view::ExtraDoors),
        })
        .to_html()
    })
}

/// The composer and the timeline tabs, drawn by the crate the client draws with.
///
/// The controls are the real ones, with verbs that do nothing until the wasm mount
/// brings the editor and the account's own SDK. The tab strip takes the route's path,
/// not the request's: Home lives at `/`, even when the reader asked for `/home`.
fn timeline_header(active: &str) -> String {
    use leptos::prelude::*;

    let here = active.to_owned();
    Owner::new().with(|| {
        nodus_view::pages::compose::inline::ComposerPreview(
            nodus_view::pages::compose::inline::preview::ComposerPreviewProps {
                active: Signal::derive(move || here.clone()),
            },
        )
        .to_html()
    })
}

/// The screen heading, drawn by the crate the client draws with.
/// The signed-in home page, painted.
pub fn home(assets: &Assets, acct: &str, statuses: &[Status], cursor: Option<&str>) -> String {
    feed(assets, acct, "home", "Home", "/home", statuses, cursor)
}

/// A timeline page, painted: the same header, the same cards, the same island.
///
/// `which` is the client's own name for the timeline, so the island it adopts is the page
/// the reader is looking at rather than the home one.
pub fn feed(
    assets: &Assets,
    acct: &str,
    which: &str,
    title: &str,
    path: &str,
    statuses: &[Status],
    cursor: Option<&str>,
) -> String {
    let island = serde_json::to_string(&Island {
        which,
        statuses,
        cursor,
    })
    .unwrap_or_else(|_| format!("{{\"which\":\"{which}\",\"statuses\":[],\"cursor\":null}}"));
    let body = Owner::new().with(|| {
        let route = match which {
            "local" => nodus_web::shared::router::Route::Local,
            "federated" => nodus_web::shared::router::Route::Federated,
            _ => nodus_web::shared::router::Route::Home,
        };
        let card: nodus_view::components::card_renderer::CardRenderer =
            std::sync::Arc::new(|status| {
                leptos::html::div()
                    .inner_html(status_card(&status))
                    .into_any()
            });
        let header: leptos::prelude::ChildrenFn = {
            let active = if which == "home" { "/" } else { path };
            let heading = timeline_header(active);
            std::sync::Arc::new(move || leptos::html::div().inner_html(heading.clone()).into_any())
        };
        nodus_view::route::RouteView(nodus_view::route::RouteViewProps {
            runtime: crate::runtime::StaticRuntime::page(
                statuses.to_vec(),
                cursor.map(str::to_owned),
            ),
            route,
            card,
            header: Some(header),
            banner: None,
        })
        .to_html()
    });
    page(
        assets,
        &format!("{title} · {acct}"),
        &chrome(true, path, &body),
        Some(&island),
    )
}

/// A thread: what came before, the status itself, then the replies.
#[must_use]
pub fn thread_page(assets: &Assets, acct: &str, status: &Status, context: &Context) -> String {
    // The page itself is shared, so what is left here is the server's half of it: the content
    // preparation, because `inner_html` carries whatever the caller hands it and the browser
    // would only ever get the instance's own text.
    let paint = |status: &Status| -> Status {
        let mut painted = status.clone();
        match painted.reblog.as_deref_mut() {
            Some(inner) => inner.content = crate::content::prepared(&inner.content, &inner.emojis),
            None => painted.content = crate::content::prepared(&painted.content, &painted.emojis),
        }
        painted
    };
    let painted_context = Context {
        ancestors: context.ancestors.iter().map(&paint).collect(),
        descendants: context.descendants.iter().map(&paint).collect(),
    };
    crate::pages::render_thread(assets, acct, &paint(status), &painted_context)
}

/// Paints a page that is already a rendered component tree.
///
/// The client's screens and the server's screens are the same components; this is the seam
/// where the server's answer becomes a document. `title` is the browser tab's name, `inner`
/// the component's HTML, and the envelope is [`page`] either way.
pub fn page_of(
    assets: &Assets,
    acct: &str,
    path: &str,
    title: &str,
    inner: &str,
    island: Option<String>,
) -> String {
    page(
        assets,
        &format!("{title} · {acct}"),
        &chrome(true, path, inner),
        island.as_deref(),
    )
}

/// The signed-out page, and the shell a signed-in reader gets before their data arrives.
pub fn landing(assets: &Assets) -> String {
    let inner = Owner::new().with(|| nodus_view::pages::landing::LandingPage().to_html());
    page(assets, "Nodus", &chrome(false, "/", &inner), None)
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
    use leptos::prelude::*;

    if acct.is_none() {
        // The client's gate, painted: a signed-out reader gets the form the client mounts,
        // with every verb absent until wasm arrives.
        let inner = Owner::new().with(|| {
            nodus_view::pages::sign_in::SignInPage(nodus_view::pages::sign_in::SignInPageProps {
                note: None,
                host: Signal::derive(String::new),
                token: Signal::derive(String::new),
                authorizing: Signal::derive(|| false),
                pasting: Signal::derive(|| false),
                authorize_error: Signal::derive(|| None),
                token_error: Signal::derive(|| None),
                startup_error: Signal::derive(|| None),
                on_host: None,
                on_token: None,
                on_authorize: None,
                on_token_submit: None,
            })
            .to_html()
        });
        return page(
            assets,
            "Sign in · Nodus",
            &chrome(false, path, &inner),
            None,
        );
    }

    let (title, description) = describe(path);
    let (tabs, active) = tabs_for(path);
    let inner = Owner::new().with(|| {
        nodus_view::pages::shell::ShellPage(nodus_view::pages::shell::ShellPageProps {
            title: title.clone(),
            description,
            tabs,
            active,
            skeleton_count: skeleton_count(path),
            on_navigate: None,
        })
        .to_html()
    });
    page(
        assets,
        &format!("{title} · Nodus"),
        &chrome(true, path, &inner),
        None,
    )
}

/// What this address is, in the same words the screen itself uses.
fn describe(path: &str) -> (String, String) {
    let head = path.trim_start_matches('/');
    let section = head.split('/').next().unwrap_or_default();
    let (title, description) = match section {
        "messages" => ("Messages", "Direct conversations, newest first."),
        "notifications" => (
            "Notifications",
            "Mentions, boosts, favourites and follow requests.",
        ),
        "search" => (
            "Search",
            "Accounts, posts and hashtags, including remote ones.",
        ),
        "directory" => ("People", "Accounts this instance suggests."),
        "saved" => ("Saved", "Your favourites and bookmarks, newest first."),
        "scheduled" => ("Scheduled", "Posts waiting to be published."),
        "lists" => ("Lists", "Hand-picked accounts with a timeline each."),
        "compose" => (
            "Compose",
            "A post, with media, a content warning and a poll.",
        ),
        "status" => ("Conversation", "This post and the replies to it."),
        "accounts" => ("Profile", "An account and what it published."),
        "settings" => (
            "Settings",
            "Profile, tags, filters, sync and this instance.",
        ),
        "moderation" => ("Moderation", "Reports, accounts and federation decisions."),
        "admin" => (
            "Administration",
            "This instance: its server and what it publishes.",
        ),
        "feed" => (
            "Feed",
            "Posts chosen for you, from this instance and elsewhere.",
        ),
        "local" => ("Local", "Posts from accounts on this instance."),
        "federated" => (
            "Federated",
            "Posts this instance has received from elsewhere.",
        ),
        _ => ("Nodus", "A Mastodon client that keeps its state."),
    };
    (title.to_owned(), description.to_owned())
}

/// The section's own tab strip, painted so the heading does not move when the client
/// mounts. The client replaces it with the same strip, one tab marked active.
fn tabs_for(path: &str) -> (Vec<nodus_view::Tab>, String) {
    let head = path.trim_start_matches('/');
    let section = head.split('/').next().unwrap_or_default();
    let links: &[(&str, &str)] = match section {
        "settings" => &[
            ("Account", "/settings/account"),
            ("Profile", "/settings/profile"),
            ("Nodus", "/settings/nodus"),
            ("Filters", "/settings/filters"),
            ("Instance", "/settings/instance"),
        ],
        "saved" => &[
            ("Favourites", "/saved/favourites"),
            ("Bookmarks", "/saved/bookmarks"),
        ],
        "local" | "federated" | "feed" | "home" => &[
            ("Home", "/home"),
            ("Feed", "/feed"),
            ("Local", "/local"),
            ("Federated", "/federated"),
        ],
        _ => &[],
    };
    if links.is_empty() {
        return (Vec::new(), String::new());
    }
    // The same component the client mounts; the only difference is the absent `on_navigate`,
    // which turns each tab into a plain link rather than a router press.
    let tabs: Vec<nodus_view::Tab> = links
        .iter()
        .map(|(label, href)| nodus_view::Tab::new(*label, *href))
        .collect();
    let active = links
        .iter()
        .find(|(_, href)| head == href.trim_start_matches('/') || path == *href)
        .map_or_else(String::new, |(label, _)| (*label).to_owned());
    (tabs, active)
}

/// How many card frames the screen's skeleton stands in for.
///
/// The server cannot paint a thread or a notification queue — it has neither — but it can
/// paint the shape, so the client's first render lands in the space already reserved.
fn skeleton_count(path: &str) -> usize {
    let head = path.trim_start_matches('/');
    let section = head.split('/').next().unwrap_or_default();
    match section {
        "messages" | "notifications" | "search" | "directory" | "saved" | "scheduled" | "lists"
        | "local" | "federated" | "feed" | "moderation" | "admin" => 4,
        "status" => 1,
        _ => 3,
    }
}
pub fn error(assets: &Assets, message: &str) -> String {
    let inner = Owner::new().with(|| {
        nodus_view::pages::shell::TimelineError(nodus_view::pages::shell::TimelineErrorProps {
            message: message.to_owned(),
        })
        .to_html()
    });
    page(
        assets,
        "Timeline unavailable",
        &chrome(true, "/home", &inner),
        None,
    )
}

/// The shell both pages share: the app's stylesheet, its bundle, and the island.
fn page(assets: &Assets, title: &str, body: &str, island: Option<&str>) -> String {
    let stylesheet = match (&assets.stylesheet_css, &assets.stylesheet) {
        (Some(css), _) => format!("<style>{}</style>", css.replace("</style", "<\\/style")),
        (None, Some(href)) => format!("<link rel=\"stylesheet\" href=\"{}\">", escape(href)),
        (None, None) => String::new(),
    };
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
