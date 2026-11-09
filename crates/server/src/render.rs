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

use leptos::prelude::Owner;
use leptos::tachys::html::element::InnerHtmlAttribute as _;
use leptos::tachys::view::RenderHtml as _;
use leptos::tachys::view::any_view::IntoAny as _;

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

/// One status as markup: who, what, and what can be done with it.
///
/// The whole card comes from `nodus-view`, the component the client mounts, so the first
/// paint and the one after the wasm boots cannot drift. The body is rewritten here rather
/// than in the component: the emoji shortcodes are a read of the instance's emoji list, and
/// `inner_html` already carries the HTML both paints show.
pub fn status_card(status: &Status) -> String {
    let mut painted = status.clone();
    // A boost renders the boosted post under the booster's name, exactly as the client
    // does: the line above the card comes from the wrapper and everything below it from
    // `reblog`.
    match painted.reblog.as_deref_mut() {
        Some(inner) => inner.content = with_emojis(body_html(&inner.content), &inner.emojis),
        None => painted.content = with_emojis(body_html(&painted.content), &painted.emojis),
    }
    let inner = painted.reblog.as_deref().map_or(&painted, |inner| inner);
    let age = nodus_view::relative_age(&inner.created_at, now_millis());
    let author_path = format!("/accounts/{}", inner.account.id);
    let thread_path = format!("/status/{}", inner.id);

    Owner::new().with(|| {
        nodus_view::status_card::StatusCard(nodus_view::status_card::StatusCardProps {
            status: painted,
            age,
            author_path,
            thread_path,
            on_navigate: None,
            on_open: None,
            on_boost: None,
            on_favourite: None,
            on_bookmark: None,
            on_reply: None,
            before_verbs: None,
            after_verbs: None,
            on_open_media: None,
            on_vote: None,
            interactive: false,
            body: None,
            extra: None,
        })
        .to_html()
    })
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
    Owner::new().with(|| {
        nodus_view::chrome::more_button::MoreButton(
            nodus_view::chrome::more_button::MoreButtonProps {
                cursor: cursor.map(std::borrow::ToOwned::to_owned),
                loading: false,
                on_more: None,
            },
        )
        .to_html()
    })
}
#[expect(
    clippy::too_many_arguments,
    reason = "one page: assets, who, where, three pieces of copy, the body and an island"
)]
#[expect(
    clippy::needless_pass_by_value,
    reason = "the island JSON is built once, for one page"
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
    let note = body_html(&account.note);
    Owner::new().with(|| {
        nodus_view::lists::account_row::AccountRow(
            nodus_view::lists::account_row::AccountRowProps {
                account: account.clone(),
                note,
                on_open: None,
                after: None,
            },
        )
        .to_html()
    })
}
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
    let age = conversation
        .last_status
        .as_ref()
        .map_or_else(String::new, |status| {
            nodus_view::relative_age(&status.created_at, now_millis())
        });
    let card = conversation
        .last_status
        .as_ref()
        .map(status_card)
        .unwrap_or_default();
    let last: Option<std::sync::Arc<dyn Fn() -> leptos::prelude::AnyView + Send + Sync>> =
        Some(std::sync::Arc::new(move || {
            // `Fn`, not `FnOnce`: the closure may be called more than once.
            leptos::html::div().inner_html(card.clone()).into_any()
        }));
    Owner::new().with(|| {
        nodus_view::lists::conversation_row::ConversationRow(
            nodus_view::lists::conversation_row::ConversationRowProps {
                conversation: conversation.clone(),
                age,
                on_open: None,
                on_read: None,
                status: last,
            },
        )
        .to_html()
    })
}
pub fn list_row(list: &List) -> String {
    Owner::new().with(|| {
        nodus_view::lists::list_row::ListRow(nodus_view::lists::list_row::ListRowProps {
            list: list.clone(),
            selected: false,
            on_select: None,
            children: None,
        })
        .to_html()
    })
}
pub fn scheduled_row(scheduled: &ScheduledStatus) -> String {
    let age = nodus_view::relative_age(&scheduled.scheduled_at, now_millis());
    Owner::new().with(|| {
        nodus_view::lists::scheduled_row::ScheduledRow(
            nodus_view::lists::scheduled_row::ScheduledRowProps {
                scheduled: scheduled.clone(),
                age,
                on_cancel: None,
            },
        )
        .to_html()
    })
}
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
    let count = match section {
        "messages" | "notifications" | "search" | "directory" | "saved" | "scheduled" | "lists"
        | "local" | "federated" | "feed" | "moderation" | "admin" => 4,
        "status" => 1,
        _ => 3,
    };
    Owner::new().with(|| {
        nodus_view::chrome::skeleton::Skeleton(nodus_view::chrome::skeleton::SkeletonProps {
            count,
        })
        .to_html()
    })
}
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
