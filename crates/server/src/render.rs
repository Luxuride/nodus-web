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

use nodus_instance::model::{
    Account, Context, Conversation, List, MediaKind, Notification, ScheduledStatus, Status,
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

/// One status as markup: who, when, what.
/// The attachment grid: images inline, video and audio with a player, and a one-picture
/// post given the full width so it is not forced into a half-empty pair.
fn media_grid(status: &Status, blurred: bool) -> String {
    // The same veil the client draws, inline rather than as a utility class: the gate must
    // hold even before the stylesheet matching it arrives, and the scale is required because
    // a blur samples outside the element and would otherwise eat its own edges.
    let veil = if blurred {
        "filter: blur(18px); transform: scale(1.08);"
    } else {
        ""
    };
    let mut media = String::new();
    for attachment in &status.media_attachments {
        let href = attachment
            .url
            .as_deref()
            .or(attachment.preview_url.as_deref())
            .unwrap_or_default();
        if href.is_empty() {
            continue;
        }
        let preview = attachment
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
                "<li class=\"self-start overflow-hidden rounded-box\"><video class=\"max-h-[32rem] w-full rounded-box bg-base-300\" controls playsinline preload=\"metadata\" loop{preview} src=\"{href}\"></video></li>"
            ),
            MediaKind::Audio => format!(
                "<li class=\"self-start overflow-hidden rounded-box\"><audio class=\"w-full\" controls preload=\"metadata\" src=\"{href}\"></audio></li>"
            ),
            MediaKind::Image | MediaKind::Unknown => {
                // A tile with no published dimensions still reserves its box: the client
                // falls back to 16:9, and a server paint that did not would push the rest of
                // the card down when the file arrives.
                let box_ = square(attachment);
                format!(
                    "<li class=\"self-start overflow-hidden rounded-box\"><img class=\"h-auto w-full rounded-box bg-base-300 object-cover\" style=\"{veil}{dimensions}\" aria-hidden=\"{blurred}\" loading=\"lazy\" decoding=\"async\" referrerpolicy=\"no-referrer\" width=\"{}\" height=\"{}\"{alt} src=\"{href}\"></li>",
                    box_.0, box_.1,
                    dimensions = box_.2
                )
            }
        };
        media.push_str(&element);
    }
    if media.is_empty() {
        String::new()
    } else {
        format!(
            "<ul class=\"{} gap-2\">{media}</ul>",
            if status.media_attachments.len() == 1 {
                "mt-1 grid grid-cols-1 items-start"
            } else {
                "mt-1 grid grid-cols-2 items-start"
            }
        )
    }
}

/// The box a media tile reserves: published dimensions when the instance has them, 16:9 when
/// it does not. Returned as `(width, height, extra style)` so the markup states both.
fn square(attachment: &nodus_instance::model::MediaAttachment) -> (u32, u32, String) {
    match (attachment.meta_width, attachment.meta_height) {
        (Some(width), Some(height)) if width > 0 && height > 0 => (width, height, String::new()),
        _ => (1600, 900, "aspect-ratio: 16 / 9;".to_owned()),
    }
}

/// "Show 1 attachment" or "Show 3 attachments", as the client says it.
fn attachments_label(status: &Status) -> String {
    let count = status.media_attachments.len();
    if count == 1 {
        "Show 1 attachment".to_owned()
    } else {
        format!("Show {count} attachments")
    }
}

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
    let name = if inner.account.display_name.is_empty() {
        inner.account.acct.clone()
    } else {
        inner.account.display_name.clone()
    };
    let grid = if inner.media_attachments.is_empty() {
        String::new()
    } else if inner.sensitive {
        // Not a convenience: painting the sharp image and hiding it once the client boots is
        // exactly the flash of sensitive content a reader did not consent to see.
        format!(
            "<div class=\"relative mt-1 overflow-hidden rounded-box\">{}<button type=\"button\" class=\"absolute inset-0 flex items-center justify-center\"><span class=\"rounded-box bg-base-300/90 px-3 py-1.5 text-center shadow-sm\"><span class=\"block text-sm font-medium\">Sensitive content</span><span class=\"block text-xs opacity-70\">{}</span></span></button></div>",
            media_grid(inner, true),
            escape(&attachments_label(inner))
        )
    } else {
        media_grid(inner, false)
    };
    let avatar = if inner.account.avatar.is_empty() {
        String::new()
    } else {
        format!(
            "<img class=\"h-full w-full object-cover\" width=\"40\" height=\"40\" decoding=\"async\" loading=\"lazy\" referrerpolicy=\"no-referrer\" alt=\"\" src=\"{}\">",
            escape(&inner.account.avatar)
        )
    };
    format!(
        "<li class=\"card border border-base-300 bg-base-100 shadow-sm\">\
           <div class=\"card-body gap-3 p-4\">{boosted}\
             <header class=\"flex items-center gap-3\">\
               <a class=\"avatar\" href=\"/accounts/{id}\">\
                 <div class=\"w-10 rounded-full bg-base-300 overflow-hidden\">{avatar}</div>\
               </a>\
               <div class=\"min-w-0 flex-1\">\
                 <a class=\"block truncate font-semibold leading-tight hover:underline\" href=\"/accounts/{id}\">{name}</a>\
                 <a class=\"block truncate text-sm opacity-60 hover:underline\" href=\"/accounts/{id}\">{handle}</a>\
               </div>\
               <a class=\"text-xs whitespace-nowrap opacity-50 hover:underline\" href=\"/status/{sid}\">{when}</a>\
             </header>\
             <div class=\"whitespace-pre-line text-sm leading-relaxed\">{body}</div>{grid}\
             <footer class=\"flex items-center justify-between gap-2 border-t border-base-200 pt-2\">\
               <span class=\"text-xs opacity-60\">{replies} replies</span>\
               <div class=\"flex items-center gap-1\">\
                 <a class=\"btn btn-ghost btn-sm\" href=\"/status/{sid}\"><span>\u{21a9}</span><span class=\"ml-1\">Reply</span></a>\
                 <span class=\"btn btn-ghost btn-sm\"><span>\u{21c4}</span><span class=\"ml-1 tabular-nums\">{reblogs}</span></span>\
                 <span class=\"btn btn-ghost btn-sm\"><span>\u{2606}</span><span class=\"ml-1 tabular-nums\">{favourites}</span></span>\
                 <span class=\"btn btn-ghost btn-sm\"><span>\u{1f516}</span></span>\
               </div>\
             </footer>\
           </div>\
         </li>",
        id = escape(&inner.account.id),
        name = escape(&name),
        handle = escape(&inner.account.acct),
        sid = escape(&inner.id),
        when = escape(&stamp(&inner.created_at)),
        body = body_html(&inner.content),
        replies = inner.replies_count,
        reblogs = inner.reblogs_count,
        favourites = inner.favourites_count,
    )
}

/// The timestamp the server paints. The client replaces it with a relative time on mount;
/// both are `text-xs` in the header, so the row keeps its height either way.
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
    .unwrap_or_else(|_| {
        format!("{{\"which\":\"{which}\",\"statuses\":[],\"cursor\":null}}")
    });
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
                "<div class=\"mx-auto flex w-full max-w-2xl flex-col gap-3\">{header}\
                   <ul class=\"flex flex-col gap-2\">{cards}</ul>{more}\
                 </div>",
                header = timeline_header()
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
                "<div class=\"mx-auto flex w-full max-w-2xl flex-col gap-3\">{header}\
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
    let ancestors = context.ancestors.iter().map(status_card).collect::<String>();
    let replies = context.descendants.iter().map(status_card).collect::<String>();
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
                "<div class=\"mx-auto flex w-full max-w-2xl flex-col gap-3\">\
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
            &format!(
                "<div class=\"mx-auto flex w-full max-w-2xl flex-col gap-3\">\
                   <header class=\"flex flex-col gap-1\">\
                     <h1 class=\"text-lg font-semibold\">{heading}</h1>\
                     <p class=\"text-sm opacity-70\">{description}</p>\
                   </header>{inner}\
                 </div>",
                heading = escape(heading),
                description = escape(description)
            ),
        ),
        island.as_deref(),
    )
}

/// A list of rows, or the sentence that says the list is empty.
pub fn rows(items: &[String], empty: &str) -> String {
    if items.is_empty() {
        return format!(
            "<p class=\"text-sm opacity-60\">{}</p>",
            escape(empty)
        );
    }
    format!(
        "<ul class=\"flex flex-col gap-2\">{}</ul>",
        items.concat()
    )
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
    let kind = match notification.kind {
        nodus_instance::model::NotificationKind::Follow => "followed you",
        nodus_instance::model::NotificationKind::FollowRequest => "asked to follow you",
        nodus_instance::model::NotificationKind::Reblog => "boosted your post",
        nodus_instance::model::NotificationKind::Favourite => "favourited your post",
        nodus_instance::model::NotificationKind::Mention => "mentioned you",
        nodus_instance::model::NotificationKind::Poll => "a poll you voted in ended",
        nodus_instance::model::NotificationKind::Status => "posted",
        nodus_instance::model::NotificationKind::AdminSignUp => "signed up",
        nodus_instance::model::NotificationKind::AdminReport => "a report was filed",
        nodus_instance::model::NotificationKind::Unknown => "sent you something",
    };
    let status = notification
        .status
        .as_ref()
        .map(status_card)
        .unwrap_or_default();
    format!(
        "<li class=\"flex flex-col gap-2\">\
           <p class=\"text-sm\"><a class=\"font-medium hover:underline\" href=\"/accounts/{id}\">{name}</a> \
             <span class=\"opacity-70\">{kind}</span></p>{status}\
         </li>",
        id = escape(&notification.account.id),
        name = escape(&notification.account.acct),
        kind = escape(kind)
    )
}

/// One conversation: everyone in it, and its last message.
#[must_use]
pub fn conversation_row(conversation: &Conversation) -> String {
    let who = conversation
        .accounts
        .first()
        .map_or_else(|| "a conversation".to_owned(), |account| account.acct.clone());
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
                    "<section class=\"mx-auto w-full max-w-md px-4 py-10\">\
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
                "<div class=\"mx-auto flex w-full max-w-2xl flex-col gap-3\">\
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
        "lists" => (
            "Lists",
            "Hand-picked accounts with a timeline each.",
        ),
        "compose" => ("Compose", "A post, with media, a content warning and a poll."),
        "status" => ("Conversation", "This post and the replies to it."),
        "accounts" => ("Profile", "An account and what it published."),
        "settings" => ("Settings", "Profile, tags, filters, sync and this instance."),
        "moderation" => ("Moderation", "Reports, accounts and federation decisions."),
        "admin" => ("Administration", "This instance: its server and what it publishes."),
        "feed" => ("Feed", "Posts chosen for you, from this instance and elsewhere."),
        "local" => ("Local", "Posts from accounts on this instance."),
        "federated" => ("Federated", "Posts this instance has received from elsewhere."),
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
            if active.is_empty() { "" } else { " aria-selected=\"true\"" },
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
        "messages" | "notifications" | "search" | "directory" | "saved" | "scheduled"
        | "lists" | "local" | "federated" | "feed" | "moderation" | "admin" => 4,
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
