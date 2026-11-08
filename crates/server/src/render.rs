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

use leptos::prelude::{Owner, Signal};
use leptos::tachys::view::RenderHtml as _;

use nodus_instance::model::{
    Account, Context, Conversation, List, Notification, ScheduledStatus, Status,
};

use crate::html::body as body_html;

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
    /// Whether the files this build discovered are still the ones on disk.
    ///
    /// `Caddy` serves `/srv/nodus` from the filesystem, so a static deploy replaces both the
    /// script and the wasm while this process keeps serving. Without this check the page
    /// keeps naming the previous bundle, the browser asks for a file that no longer exists,
    /// and the wasm never boots — the document stays server-painted and looks like SSR
    /// having replaced CSR.
    #[must_use]
    pub fn stale(&self, dist: &std::path::Path) -> bool {
        !dist.join(&self.wasm).is_file() || !dist.join(&self.script).is_file()
    }

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

/// The author line above a status, drawn by the component the client draws with.
///
/// The addresses are the server's own routes and the age comes from the clock this
/// process reads once per card; with no `on_navigate` the anchors stay plain links, which
/// is what the server's reader gets anyway.
fn status_header(status: &Status) -> String {
    let age = nodus_view::relative_age(&status.created_at, now_millis());
    let author_path = format!("/accounts/{}", status.account.id);
    let thread_path = format!("/status/{}", status.id);
    let status = status.clone();
    Owner::new().with(|| {
        nodus_view::status_header::StatusHeader(nodus_view::status_header::StatusHeaderProps {
            status,
            age,
            author_path,
            thread_path,
            on_navigate: None,
        })
        .to_html()
    })
}

/// The action row under a status, drawn by the component the client draws with.
///
/// The verbs have no callbacks here, so they render as the client's boxes without the
/// press; the reply link keeps its `href`.
fn status_footer(status: &Status) -> String {
    let edited = status.edited_at.is_some();
    let status = status.clone();
    Owner::new().with(|| {
        nodus_view::status_footer::StatusFooter(nodus_view::status_footer::StatusFooterProps {
            status,
            edited,
            on_reply: None,
            on_boost: None,
            on_favourite: None,
            on_bookmark: None,
            before_verbs: None,
            after_verbs: None,
        })
        .to_html()
    })
}

/// The attachment grid: images inline, video and audio with a player, and a one-picture
/// post given the full width so it is not forced into a half-empty pair.
fn media_grid(status: &Status, blurred: bool) -> String {
    if status.media_attachments.is_empty() {
        return String::new();
    }
    // The grid is a component, not a string: the client mounts the same one, so the tiles,
    // the reserved boxes and the veil cannot drift apart between the two paints.
    let items = status.media_attachments.clone();
    Owner::new().with(|| {
        nodus_view::media_grid::MediaGrid(nodus_view::media_grid::MediaGridProps {
            items: Signal::derive(move || items.clone()),
            hidden: blurred,
            interactive: false,
            on_open: None,
        })
        .to_html()
    })
}

/// One status as markup: who, when, what.
///
/// The author line and the action row come from `nodus-view`, the same components the
/// client mounts, so the first paint and the one after the wasm boots cannot drift.
pub fn status_card(status: &Status) -> String {
    // A boost renders the boosted post under the booster's name, exactly as the client
    // does: the wrapper carries nothing but the line that says who boosted it.
    let (boosted, inner) = match status.reblog.as_deref() {
        Some(inner) => (
            format!(
                "<p class=\"text-xs opacity-60\">{} boosted</p>",
                escape(&status.account.acct)
            ),
            inner,
        ),
        None => (String::new(), status),
    };
    let grid = if inner.media_attachments.is_empty() {
        String::new()
    } else if inner.sensitive {
        // Not a convenience: painting the sharp image and hiding it once the client boots is
        // exactly the flash of sensitive content a reader did not consent to see.
        format!(
            "<div class=\"relative mt-1 overflow-hidden rounded-box\">{}<button type=\"button\" class=\"absolute inset-0 flex items-center justify-center\"><span class=\"rounded-box bg-base-300/90 px-3 py-1.5 text-center shadow-sm\"><span class=\"block text-sm font-medium\">Sensitive content</span><span class=\"block text-xs opacity-70\">{}</span></span></button></div>",
            media_grid(inner, true),
            escape(&nodus_view::media::attachments_label(inner))
        )
    } else {
        media_grid(inner, false)
    };
    let header = status_header(inner);
    let footer = status_footer(inner);
    format!(
        "<li class=\"card border border-base-300 bg-base-100 shadow-sm\">\
           <div class=\"card-body gap-3 p-4\">{boosted}{header}\
             <div class=\"whitespace-pre-line text-sm leading-relaxed\">{body}</div>{grid}{footer}\
           </div>\
         </li>",
        body = with_emojis(body_html(&inner.content), &inner.emojis),
    )
}

/// The absolute timestamp the server paints, with the date and the minute.
///
/// Only the scheduled queue uses it: a post that has not gone out yet is read as a time
/// and a day, where "in 3h" would be a moving target. Statuses take their age from
/// [`nodus_view::relative_age`], the client's own wording.
fn stamp(rfc3339: &str) -> String {
    rfc3339
        .get(..16)
        .map(|head| head.replace('T', " "))
        .unwrap_or_default()
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
    use nodus_view::icons::Icon;

    /// The moderation and administration rows.
    #[must_use]
    pub fn rail_rows() -> AnyView {
        let row = |label: &'static str, href: &'static str, hint: &'static str| {
            view! {
                <li>
                    <a
                        class="flex w-full items-center gap-2 font-medium"
                        href=href
                        title=hint
                    >
                        <Icon icon=Icon::Shield/>
                        <span class="truncate">{label}</span>
                    </a>
                </li>
            }
        };
        view! {
            {row("Moderation", "/moderation", "The queue, the accounts and what the server allows")}
            {row("Administration", "/admin", "What this instance publishes and allows")}
        }
        .into_any()
    }
}

fn chrome(signed_in: bool, path: &str, inner: &str) -> String {
    use leptos::prelude::{IntoAny, Owner, Signal};
    use leptos::tachys::html::element::InnerHtmlAttribute as _;
    use leptos::tachys::view::RenderHtml as _;

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

/// The composer and the timeline tabs, at the size the client gives them.
/// The composer and the tab strip, drawn by the crate the client draws with.
///
/// The textarea and the buttons here are inert on purpose: the wasm mount replaces them
/// before the reader can type, and a control that pretends to work is worse than one that
/// plainly waits.
fn timeline_header(path: &str) -> String {
    use leptos::prelude::*;

    let here = path.to_owned();
    Owner::new().with(|| {
        view! {
            <div class="flex flex-col gap-2">
                <nodus_view::ComposerCard>
                    <textarea
                        class="textarea textarea-ghost min-h-20 w-full resize-none p-1 text-[15px]"
                        aria-label="Write a post"
                        placeholder="What is on your mind?"
                    ></textarea>
                    <div class="flex flex-wrap items-center gap-2">
                        <span class="btn btn-outline btn-sm gap-1 border-dashed">"+"</span>
                        <span class="select select-bordered select-sm">"Public"</span>
                        <span class="ml-auto text-xs opacity-50">"0/500"</span>
                        <span class="btn btn-sm btn-primary">"Post"</span>
                    </div>
                </nodus_view::ComposerCard>
                <nodus_view::TimelineTabs active=Signal::derive(move || here.clone())/>
            </div>
        }
        .to_html()
    })
}

/// The screen heading, drawn by the crate the client draws with.
/// Why this timeline exists, in the words the client uses for the same screen.
fn timeline_description(which: &str) -> &'static str {
    match which {
        "local" => "Posts from accounts on this instance.",
        "federated" => "Posts this instance has received from elsewhere.",
        "feed" => "People this instance suggests, and what they are posting.",
        _ => "Posts from the accounts you follow, newest first.",
    }
}

fn page_header(title: &str, description: &str) -> String {
    use leptos::prelude::*;

    let title = title.to_owned();
    let description = description.to_owned();
    Owner::new().with(|| {
        nodus_view::PageHeader(nodus_view::PageHeaderProps { title, description }).to_html()
    })
}

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
    let cards = statuses.iter().map(status_card).collect::<String>();
    let island = serde_json::to_string(&Island {
        which,
        statuses,
        cursor,
    })
    .unwrap_or_else(|_| format!("{{\"which\":\"{which}\",\"statuses\":[],\"cursor\":null}}"));
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
        &format!("{title} · {acct}"),
        &chrome(
            true,
            path,
            &format!(
                "{heading}{header}\
                   <ul class=\"flex flex-col gap-2\">{cards}</ul>{more}",
                heading = page_header(title, timeline_description(which)),
                header = timeline_header(path)
            ),
        ),
        Some(&island),
    )
}

/// A profile: who they are, then what they posted.
#[must_use]
pub fn profile_page(
    assets: &Assets,
    acct: &str,
    account: &Account,
    statuses: &[Status],
    cursor: Option<&str>,
) -> String {
    let name = if account.display_name.trim().is_empty() {
        account.acct.clone()
    } else {
        account.display_name.clone()
    };
    let avatar = if account.avatar.is_empty() {
        String::new()
    } else {
        format!(
            "<img class=\"h-16 w-16 rounded-full object-cover\" width=\"64\" height=\"64\" \
               decoding=\"async\" referrerpolicy=\"no-referrer\" alt=\"\" src=\"{}\">",
            escape(&account.avatar)
        )
    };
    let header = format!(
        "<header class=\"card border border-base-300 bg-base-100 shadow-sm\">\
           <div class=\"card-body gap-3 p-4\">\
             <div class=\"flex items-center gap-3\">{avatar}\
               <div class=\"min-w-0\">\
                 <h1 class=\"text-lg font-semibold\">{name}</h1>\
                 <p class=\"truncate text-sm opacity-60\">@{acct}</p>\
               </div>\
             </div>\
             <p class=\"text-sm opacity-70\">{counts}</p>\
             <div class=\"text-sm\">{note}</div>\
           </div>\
         </header>",
        name = escape(&name),
        acct = escape(&account.acct),
        counts = escape(&format!(
            "{} posts · {} followers · {} following",
            account.statuses_count, account.followers_count, account.following_count
        )),
        note = body_html(&account.note)
    );
    let cards = statuses.iter().map(status_card).collect::<String>();
    page(
        assets,
        &format!("{} · {acct}", account.acct),
        &chrome(
            true,
            &format!("/accounts/{}", account.id),
            &format!(
                "<div class=\"mx-auto flex w-full flex-col gap-3 px-2 max-w-2xl\">{header}\
                   <ul class=\"flex flex-col gap-2\">{cards}</ul>{more}\
                 </div>",
                more = more_button(cursor)
            ),
        ),
        None,
    )
}

/// A thread: what came before, the status itself, then the replies.
#[must_use]
pub fn thread_page(assets: &Assets, acct: &str, status: &Status, context: &Context) -> String {
    let ancestors = context
        .ancestors
        .iter()
        .map(status_card)
        .collect::<String>();
    let replies = context
        .descendants
        .iter()
        .map(status_card)
        .collect::<String>();
    let replies = if replies.is_empty() {
        "<p class=\"text-sm opacity-60\">No replies yet.</p>".to_owned()
    } else {
        format!(
            "<h2 class=\"text-sm font-semibold\">Replies</h2>\
             <ul class=\"flex flex-col gap-2\">{replies}</ul>"
        )
    };
    page(
        assets,
        &format!("Thread · {acct}"),
        &chrome(
            true,
            &format!("/status/{}", status.id),
            &format!(
                "<div class=\"mx-auto flex w-full flex-col gap-3 px-2 max-w-2xl\">\
                   <ul class=\"flex flex-col gap-2\">{ancestors}</ul>\
                   <ul class=\"flex flex-col gap-2\">{focused}</ul>{replies}\
                 </div>",
                focused = status_card(status)
            ),
        ),
        None,
    )
}

/// The "Load more" button the client also draws, or nothing on the last page.
fn more_button(cursor: Option<&str>) -> String {
    cursor
        .map(|cursor| {
            format!(
                "<button class=\"btn btn-ghost btn-sm w-full\" data-cursor=\"{}\">Load more</button>",
                escape(cursor)
            )
        })
        .unwrap_or_default()
}

/// A page whose body is a list of things that are not statuses.
///
/// The chrome, the heading and the description come from the client's own words, so a reader
/// who lands here and a reader who navigates here see the same page.
#[expect(
    clippy::needless_pass_by_value,
    reason = "the island JSON is built once, for one page"
)]
#[expect(
    clippy::too_many_arguments,
    reason = "a page is a title, a description, a body and its island; a struct would only name them twice"
)]
pub fn list_page(
    assets: &Assets,
    acct: &str,
    path: &str,
    title: &str,
    heading: &str,
    description: &str,
    inner: &str,
    island: Option<String>,
) -> String {
    page(
        assets,
        &format!("{title} · {acct}"),
        &chrome(
            true,
            path,
            &format!("{}{inner}", page_header(heading, description)),
        ),
        island.as_deref(),
    )
}

/// A list of rows, or the sentence that says the list is empty.
pub fn rows(items: &[String], empty: &str) -> String {
    if items.is_empty() {
        return format!("<p class=\"text-sm opacity-60\">{}</p>", escape(empty));
    }
    format!("<ul class=\"flex flex-col gap-2\">{}</ul>", items.concat())
}

/// One account as a row: who they are, and the one line of self-description they wrote.
#[must_use]
pub fn account_row(account: &Account) -> String {
    let name = if account.display_name.trim().is_empty() {
        account.acct.clone()
    } else {
        account.display_name.clone()
    };
    let avatar = if account.avatar.is_empty() {
        String::new()
    } else {
        format!(
            "<img class=\"h-10 w-10 rounded-full object-cover\" width=\"40\" height=\"40\" \
               decoding=\"async\" loading=\"lazy\" referrerpolicy=\"no-referrer\" alt=\"\" src=\"{}\">",
            escape(&account.avatar)
        )
    };
    let note = body_html(&account.note);
    let note = if note.trim().is_empty() {
        String::new()
    } else {
        format!("<p class=\"line-clamp-2 text-sm opacity-70\">{note}</p>")
    };
    format!(
        "<li class=\"card border border-base-300 bg-base-100 shadow-sm\">\
           <div class=\"card-body flex-row items-start gap-3 p-3\">\
             <a class=\"avatar\" href=\"/accounts/{id}\">{avatar}</a>\
             <div class=\"min-w-0 flex-1\">\
               <a class=\"font-medium hover:underline\" href=\"/accounts/{id}\">{name}</a>\
               <p class=\"truncate text-xs opacity-60\">@{acct}</p>{note}\
             </div>\
           </div>\
         </li>",
        id = escape(&account.id),
        acct = escape(&account.acct),
        name = escape(&name)
    )
}

/// One notification: what happened, to whom, and the status when the event carries one.
#[must_use]
pub fn notification_row(notification: &Notification) -> String {
    use leptos::prelude::*;

    let age = nodus_view::relative_age(&notification.created_at, now_millis());
    let body = notification.status.as_ref().map(status_card).map(|html| {
        let body = std::sync::Arc::new(move || {
            leptos::html::ul()
                .class("flex flex-col gap-2")
                .inner_html(html.clone())
                .into_any()
        });
        body as ChildrenFn
    });
    let notification = notification.clone();

    Owner::new().with(|| {
        nodus_view::NotificationCard(nodus_view::NotificationCardProps {
            notification,
            age,
            body,
            extra: None,
        })
        .to_html()
    })
}

/// Milliseconds since the Unix epoch, by this host's clock.
fn now_millis() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0.0, |elapsed| elapsed.as_secs_f64() * 1_000.0)
}

/// One conversation: everyone in it, and its last message.
#[must_use]
pub fn conversation_row(conversation: &Conversation) -> String {
    let who = conversation.accounts.first().map_or_else(
        || "a conversation".to_owned(),
        |account| account.acct.clone(),
    );
    let rest = conversation.accounts.len().saturating_sub(1);
    let name = if rest == 0 {
        who
    } else {
        format!("{who} and {rest} more")
    };
    let last = conversation
        .last_status
        .as_ref()
        .map(status_card)
        .unwrap_or_default();
    let unread = if conversation.unread {
        "<span class=\"badge badge-primary badge-xs\">unread</span>"
    } else {
        ""
    };
    format!(
        "<li class=\"flex flex-col gap-2\"><p class=\"text-sm font-medium\">{}\n{unread}</p>{last}</li>",
        escape(&name)
    )
}

/// One list: its title, and how many people it holds when the instance says.
#[must_use]
pub fn list_row(list: &List) -> String {
    format!(
        "<li class=\"card border border-base-300 bg-base-100 shadow-sm\">\
           <div class=\"card-body flex-row items-center justify-between gap-3 p-3\">\
             <a class=\"font-medium hover:underline\" href=\"/lists/{}\">{}</a>\
             <span class=\"text-xs opacity-60\">{}</span>\
           </div>\
         </li>",
        escape(&list.id),
        escape(&list.title),
        escape(&list.replies_policy)
    )
}

/// One scheduled post: when it goes out, and what it will say.
#[must_use]
pub fn scheduled_row(scheduled: &ScheduledStatus) -> String {
    format!(
        "<li class=\"card border border-base-300 bg-base-100 shadow-sm\">\
           <div class=\"card-body gap-1 p-3\">\
             <p class=\"text-xs opacity-60\">{}</p>\
             <p class=\"whitespace-pre-line text-sm\">{}</p>\
           </div>\
         </li>",
        escape(&stamp(&scheduled.scheduled_at)),
        escape(&scheduled.summary())
    )
}

/// The signed-out page: the app's own front door, painted without data.
pub fn landing(assets: &Assets) -> String {
    page(
        assets,
        "Nodus",
        &chrome(
            false,
            "/",
            "<div class=\"mx-auto flex w-full flex-col gap-3 px-2 max-w-2xl\">\
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
    if acct.is_none() {
        // The client's gate, painted: a signed-out reader never gets a skeleton of a page
        // they cannot see, and the card they *do* get is the one the client renders.
        let (_, description) = describe(path);
        return page(
            assets,
            "Sign in · Nodus",
            &chrome(
                false,
                path,
                &format!(
                    "<section class=\"mx-auto flex w-full flex-col gap-3 px-2 max-w-md\">\
                       <div class=\"card border border-base-300 bg-base-100 shadow-sm\">\
                         <div class=\"card-body gap-4\">\
                           <h1 class=\"card-title\">Sign in</h1>\
                           <p class=\"text-sm font-medium\">{}</p>\
                           <p class=\"text-sm opacity-70\">Authorize this app on your instance. Approval happens on your instance's own page, and the access token it returns can be revoked there at any time.</p>\
                           <span class=\"input input-bordered flex w-full items-center opacity-60\">mastodon.social</span>\
                           <span class=\"btn btn-primary mt-2\">Sign in</span>\
                         </div>\
                       </div>\
                     </section>",
                    escape(&description)
                ),
            ),
            None,
        );
    }
    let (title, description) = describe(path);
    page(
        assets,
        &format!("{title} · Nodus"),
        &chrome(
            acct.is_some(),
            path,
            &format!(
                "<div class=\"mx-auto flex w-full flex-col gap-3 px-2 max-w-2xl\">\
                   <div class=\"flex flex-col gap-1\">\
                     <h1 class=\"text-xl font-semibold tracking-tight\">{}</h1>\
                     <p class=\"text-sm opacity-70\">{}</p>\
                   </div>{tabs}{body}\
                 </div>",
                escape(&title),
                escape(&description),
                tabs = tabs_for(path),
                body = skeleton(path)
            ),
        ),
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
fn tabs_for(path: &str) -> String {
    let head = path.trim_start_matches('/');
    let section = head.split('/').next().unwrap_or_default();
    let tabs: &[(&str, &str)] = match section {
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
    if tabs.is_empty() {
        return String::new();
    }
    let mut items = String::new();
    for (label, href) in tabs {
        let active = if head == href.trim_start_matches('/') || path == *href {
            " tab-active"
        } else {
            ""
        };
        let _ = write!(
            items,
            "<a role=\"tab\" class=\"tab{active}\" href=\"{}\"{}>{}</a>",
            escape(href),
            if active.is_empty() {
                ""
            } else {
                " aria-selected=\"true\""
            },
            escape(label)
        );
    }
    format!("<div role=\"tablist\" class=\"tabs tabs-box\">{items}</div>")
}

/// A skeleton of the screen, drawn at the sizes the real cards take.
///
/// The server cannot paint a thread or a notification queue — it has neither — but it can
/// paint the shape: the same card frames, at the same paddings, so the client's first
/// render lands in the space already reserved instead of pushing the page around.
fn skeleton(path: &str) -> String {
    let head = path.trim_start_matches('/');
    let section = head.split('/').next().unwrap_or_default();
    let rows = match section {
        "messages" | "notifications" | "search" | "directory" | "saved" | "scheduled" | "lists"
        | "local" | "federated" | "feed" | "moderation" | "admin" => 4,
        "status" => 1,
        _ => 3,
    };
    let mut cards = String::new();
    for _ in 0..rows {
        let _ = write!(
            cards,
            "<li class=\"card border border-base-300 bg-base-100 shadow-sm\">\
               <div class=\"card-body gap-3 p-4\">\
                 <div class=\"flex items-center gap-3\">\
                   <div class=\"skeleton h-10 w-10 shrink-0 rounded-full\"></div>\
                   <div class=\"flex min-w-0 flex-1 flex-col gap-1\">\
                     <div class=\"skeleton h-3 w-1/3\"></div>\
                     <div class=\"skeleton h-3 w-1/4\"></div>\
                   </div>\
                 </div>\
                 <div class=\"skeleton h-3 w-full\"></div>\
                 <div class=\"skeleton h-3 w-4/5\"></div>\
                 <div class=\"flex gap-2 border-t border-base-200 pt-2\">\
                   <div class=\"skeleton h-6 w-16\"></div>\
                   <div class=\"skeleton h-6 w-16\"></div>\
                 </div>\
               </div>\
             </li>"
        );
    }
    format!("<ul class=\"flex flex-col gap-2\">{cards}</ul>")
}

/// A page that says what went wrong and still loads the app, so the reader can sign out or
/// retry instead of staring at an empty document.
pub fn error(assets: &Assets, message: &str) -> String {
    page(
        assets,
        "Timeline unavailable",
        &chrome(
            true,
            "/home",
            &format!(
                "<div class=\"mx-auto flex w-full flex-col gap-3 px-2 max-w-2xl\">\
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

/// Draws the status's own custom emoji in its body, the same way the client does.
///
/// The server paints before any wasm runs, so without this a post with shortcodes showed
/// as `:blobcat:` until the client took over — a visible flash of text where a picture
/// belongs. Only the emoji the status itself lists are substituted: the instance's whole
/// catalogue is not needed to render one post, and an unknown shortcode stays text.
fn with_emojis(html: String, emojis: &[nodus_instance::model::CustomEmoji]) -> String {
    if emojis.is_empty() {
        return html;
    }
    let mut painted = html;
    for emoji in emojis {
        let source = if emoji.url.is_empty() {
            emoji.static_url.as_deref().unwrap_or_default()
        } else {
            emoji.url.as_str()
        };
        if source.is_empty() {
            continue;
        }
        let token = format!(":{}:", emoji.shortcode);
        let replacement = format!(
            "<img class=\"emoji inline-block h-[1.2em] w-auto align-[-0.2em]\" alt=\"{token}\" title=\"{token}\" loading=\"lazy\" decoding=\"async\" src=\"{source}\">",
            token = escape(&token),
            source = escape(source)
        );
        painted = painted.replace(&token, &replacement);
    }
    painted
}
