//! The parts of the interface that both the browser and the server draw.
//!
//! This crate exists because of one bug that kept coming back: the server painted a page
//! that looked nothing like the one the client painted a moment later, so every load moved
//! the layout under the reader. Two templates cannot be kept in sync; one component tree
//! can. Everything here is pure markup — no `web_sys`, no signals that outlive a render, no
//! router — so it compiles for `wasm32` and for the server, and `leptos::ssr` can turn it
//! into the first paint while the client turns the same tree into the live one.

pub mod icons;
pub mod chrome;
pub mod lists;
pub mod media;
pub mod media_grid;
pub mod status_footer;
pub mod status_header;

use icons::Icon;
use leptos::prelude::*;
use std::sync::Arc;

/// The drawing for a destination, by name.
///
/// The label is the key rather than a field on `Destination`, because the label is what a
/// reader sees and what the list is ordered by; a second field would be one more thing to
/// keep in step for no gain.
#[must_use]
pub fn icon_of(label: &str) -> Icon {
    match label {
        "Home" => Icon::Home,
        "Feed" => Icon::Sparkle,
        "Local" | "People" => Icon::Users,
        "Federated" => Icon::Globe,
        "Messages" => Icon::Mail,
        "Notifications" => Icon::Bell,
        "Search" => Icon::Search,
        "Saved" => Icon::Bookmark,
        "Scheduled" => Icon::Clock,
        "Lists" => Icon::List,
        "Compose" => Icon::Pen,
        "Moderation" | "Administration" => Icon::Shield,
        _ => Icon::Settings,
    }
}

/// Doors only some readers have, as a function so the same value can be drawn by both
/// paints: the server closes over server markup, the browser closes over live components.
pub type ExtraDoors = Arc<dyn Fn() -> AnyView + Send + Sync>;

/// One destination in the rail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Destination {
    /// What the reader sees.
    pub label: &'static str,
    /// Where it goes.
    pub href: &'static str,
    /// One sentence, shown as the hover title.
    pub hint: &'static str,
}

/// A named group of destinations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group {
    /// The section heading, hidden on narrow screens.
    pub title: &'static str,
    /// Its destinations, in order.
    pub destinations: &'static [Destination],
}

/// The rail the client shows, in its order.
#[must_use]
pub fn rail() -> Vec<Group> {
    vec![
        Group {
            title: "Read",
            destinations: &[
                Destination {
                    label: "Home",
                    href: "/",
                    hint: "Your timeline, the feed, and the instance's",
                },
                Destination {
                    label: "Messages",
                    href: "/messages",
                    hint: "Direct conversations",
                },
                Destination {
                    label: "Notifications",
                    href: "/notifications",
                    hint: "Mentions, boosts, favourites and follow requests",
                },
            ],
        },
        Group {
            title: "Find",
            destinations: &[
                Destination {
                    label: "Search",
                    href: "/search",
                    hint: "Accounts, posts and hashtags, including remote ones",
                },
                Destination {
                    label: "People",
                    href: "/directory",
                    hint: "Accounts this instance suggests",
                },
                Destination {
                    label: "Saved",
                    href: "/saved/favourites",
                    hint: "Your favourites and bookmarks",
                },
            ],
        },
        Group {
            title: "Library",
            destinations: &[
                Destination {
                    label: "Scheduled",
                    href: "/scheduled",
                    hint: "Posts waiting to be published",
                },
                Destination {
                    label: "Lists",
                    href: "/lists",
                    hint: "Hand-picked accounts with a timeline each",
                },
                Destination {
                    label: "Settings",
                    href: "/settings/account",
                    hint: "Profile, tags, filters, sync and this instance",
                },
            ],
        },
    ]
}

/// The rail.
///
/// The icon box is drawn at the icon's size rather than as an SVG: `view` deliberately has
/// no icon set, and the client swaps in the real paths when it takes over — the box is what
/// keeps the column from moving.
#[component]
pub fn Rail(
    /// The address being shown, so the matching destination is marked.
    #[prop(into)]
    active: Signal<String>,
    /// Doors that only some readers have: the moderation and administration ones.
    extra: Option<ExtraDoors>,
) -> impl IntoView {
    view! {
        <nav class="hidden shrink-0 xl:block xl:w-44">
            <ul class="menu w-full flex-row flex-wrap gap-1 rounded-box bg-base-100 p-2 shadow-sm xl:flex-col xl:flex-nowrap">
                {rail()
                    .into_iter()
                    .map(|group| {
                        view! {
                            <li class="menu-title hidden xl:block">{group.title}</li>
                            {group
                                .destinations
                                .iter()
                                .map(|destination| {
                                    let href = destination.href;
                                    view! {
                                        <li>
                                            <a
                                                class="flex w-full items-center gap-2 font-medium"
                                                class:menu-active=move || active.get() == href
                                                href=href
                                                title=destination.hint
                                            >
                                                <Icon icon=icon_of(destination.label)/>
                                                <span class="truncate">{destination.label}</span>
                                            </a>
                                        </li>
                                    }
                                })
                                .collect_view()}
                        }
                    })
                    .collect_view()}
                {move || extra.as_ref().map(|doors| doors())}
            </ul>
        </nav>
    }
}

#[component]
pub fn Frame(
    /// Whether the reader has a session. The rail is navigation, so it is only there for
    /// someone who can use it.
    #[prop(into)]
    signed_in: Signal<bool>,
    /// The address being shown.
    #[prop(into)]
    path: Signal<String>,
    /// The screen itself. A function, not a value, so a route change repaints the content
    /// without rebuilding the frame around it (which would drop the screen's state).
    inner: ChildrenFn,
    /// Doors only some readers have.
    extra: Option<ExtraDoors>,
) -> impl IntoView {
    let inner = StoredValue::new(inner);
    view! {
        <div class="min-h-screen bg-base-200 text-base-content">
            <Show when=move || signed_in.get()>
                <header class="sticky top-0 z-30 flex items-center justify-between gap-2 border-b border-base-300 bg-base-100 px-3 py-2 xl:hidden">
                    <a class="font-semibold tracking-tight" href="/">"Nodus"</a>
                </header>
            </Show>
            <div class="mx-auto flex w-full max-w-6xl flex-col gap-4 px-2 py-4 xl:flex-row">
                <Show when=move || signed_in.get()>
                    <Rail active=path extra=extra.clone()/>
                </Show>
                <main class="min-w-0 flex-1 pb-[calc(5.5rem+env(safe-area-inset-bottom))] xl:pb-0">
                    {move || inner.with_value(|inner| inner())}
                </main>
            </div>
        </div>
    }
}

/// A timeline the reader can switch to, as a link.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimelineLink {
    pub label: &'static str,
    pub href: &'static str,
}

/// The four timelines, in the order the tab strip shows them.
pub const TIMELINES: [TimelineLink; 4] = [
    TimelineLink {
        label: "Home",
        href: "/",
    },
    TimelineLink {
        label: "Feed",
        href: "/feed",
    },
    TimelineLink {
        label: "Local",
        href: "/local",
    },
    TimelineLink {
        label: "Federated",
        href: "/federated",
    },
];

/// The tab strip under the composer.
///
/// Links, not a tab signal: each timeline is an address, so a reload keeps the timeline the
/// reader was reading. The server draws every one of them; a reader who turned a timeline
/// off sees it here until the client takes over, which is a smaller surprise than a strip
/// that grows after hydration.
#[component]
pub fn TimelineTabs(#[prop(into)] active: Signal<String>) -> impl IntoView {
    view! {
        <div role="tablist" class="tabs tabs-box mb-2 w-full flex-wrap">
            {TIMELINES
                .iter()
                .map(|timeline| {
                    let href = timeline.href;
                    let label = timeline.label;
                    view! {
                        <a
                            role="tab"
                            class="tab"
                            href=href
                            class:tab-active=move || active.get() == href
                        >
                            {label}
                        </a>
                    }
                })
                .collect::<Vec<_>>()}
        </div>
    }
}

/// The card the composer lives in: one border, one padding, one shadow, so the server's
/// static box and the client's live one occupy the same pixels.
#[component]
pub fn ComposerCard(children: ChildrenFn) -> impl IntoView {
    let children = StoredValue::new(children);
    view! {
        <div class="card border-2 border-base-300 bg-base-100 shadow-sm has-[textarea:focus]:border-primary/50">
            <div class="card-body gap-2 p-3">{children.with_value(|children| children())}</div>
        </div>
    }
}

/// A title and one sentence of orientation.
///
/// The sentence is not decoration: a reader arriving from a link needs to know what the
/// screen is for before it finishes loading, and every empty state on the screen is cheaper
/// to read once that is on the page. It lives here because both paints open a screen with
/// it, and a heading that drifts by one opacity step is exactly the kind of difference that
/// makes a handover look like a reload.
#[component]
pub fn PageHeader(#[prop(into)] title: String, #[prop(into)] description: String) -> impl IntoView {
    view! {
        <header class="flex flex-col gap-0.5">
            <h1 class="text-lg font-semibold">{title}</h1>
            <p class="text-sm opacity-60">{description}</p>
        </header>
    }
}

/// Days from the civil date to the Unix epoch.
///
/// Howard Hinnant's `days_from_civil`, which is exact for every date the wire can carry and
/// needs no calendar crate: an age in hours does not justify a dependency.
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let shifted_month = (month + 9) % 12;
    let day_of_year = (153 * shifted_month + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

/// Seconds since the Unix epoch for an RFC 3339 stamp, or `None` when it cannot be read.
///
/// Only the shape Mastodon sends is read (`YYYY-MM-DDTHH:MM:SS`, optionally with a fractional
/// part and a `Z`/offset tail, which is ignored); a stamp that does not match answers `None`
/// so the caller can render nothing rather than a wrong age.
#[must_use]
pub fn unix_seconds(rfc3339: &str) -> Option<i64> {
    let bytes = rfc3339.as_bytes();
    if bytes.len() < 19 {
        return None;
    }
    let number = |from: usize, to: usize| -> Option<i64> { rfc3339.get(from..to)?.parse().ok() };
    let year = number(0, 4)?;
    let month = number(5, 7)?;
    let day = number(8, 10)?;
    let hour = number(11, 13)?;
    let minute = number(14, 16)?;
    let second = number(17, 19)?;
    Some(days_from_civil(year, month, day) * 86_400 + hour * 3_600 + minute * 60 + second)
}

/// A coarse, honest age: exact timestamps are one click away at the instance.
///
/// Parsing is deliberately shallow and the answer is deliberately imprecise: the server and
/// the browser each compute it from the same instant, and a difference of seconds is fine
/// while a difference of *shape* ("3h" against "3 hours ago") reads as a page that changed
/// its mind. So both paints call this one function.
#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "minutes since a past instant, clamped non-negative, fit a u32 for centuries"
)]
pub fn relative_age(created_at: &str, now_millis: f64) -> String {
    let Some(stamp) = unix_seconds(created_at) else {
        return String::new();
    };
    #[expect(
        clippy::cast_precision_loss,
        reason = "seconds since 1970 in a timestamp are exact in f64 well past this century"
    )]
    let published = stamp as f64;
    let elapsed_minutes = (now_millis / 60_000.0) - published / 60.0;
    let minutes = if elapsed_minutes <= 0.0 {
        0
    } else {
        elapsed_minutes as u32
    };
    match minutes {
        0 => "now".to_owned(),
        1..=59 => format!("{minutes}m"),
        60..=1439 => format!("{}h", minutes / 60),
        _ => format!("{}d", minutes / (60 * 24)),
    }
}

/// The name to show for an account: the display name, or the handle when it is empty.
#[must_use]
pub fn display_name(display_name: &str, acct: &str) -> String {
    if display_name.trim().is_empty() {
        acct.to_owned()
    } else {
        display_name.to_owned()
    }
}

/// What kind of attention a notification was.
#[must_use]
pub fn describe_notification(kind: nodus_instance::model::NotificationKind) -> &'static str {
    use nodus_instance::model::NotificationKind;
    match kind {
        NotificationKind::Follow => "followed you",
        NotificationKind::FollowRequest => "asked to follow you",
        NotificationKind::Reblog => "boosted your post",
        NotificationKind::Favourite => "favourited your post",
        NotificationKind::Mention => "mentioned you",
        NotificationKind::Poll => "A poll you voted in has ended",
        NotificationKind::Status => "posted",
        NotificationKind::AdminSignUp => "A new account signed up",
        NotificationKind::AdminReport => "A report needs review",
        NotificationKind::Unknown => "Something happened",
    }
}

/// One notification: who, what, when, and whatever the caller renders beneath.
///
/// `body` is the status the instance inlined (a card on both paints) and `extra` is the
/// client's follow-request controls, which the server has no business drawing. Everything
/// else — the avatar, the phrase, the age, the card — is here, so the two paints cannot
/// drift apart on it.
#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "a Leptos component receives its props by value"
)]
pub fn NotificationCard(
    #[prop(into)] notification: nodus_instance::model::Notification,
    #[prop(into)] age: String,
    body: Option<ChildrenFn>,
    extra: Option<ChildrenFn>,
) -> impl IntoView {
    let who = notification.account.clone();
    let heading = display_name(&who.display_name, &who.acct);
    let said = describe_notification(notification.kind);
    let profile = format!("/accounts/{}", who.id);
    let created_at = notification.created_at.clone();
    let avatar = who.avatar.clone();
    let body = StoredValue::new(body);
    let extra = StoredValue::new(extra);

    view! {
        <li class="card border border-base-300 bg-base-100 shadow-sm">
            <div class="card-body gap-2 p-4">
                <div class="flex items-center gap-2">
                    <a class="avatar" href=profile.clone()>
                        <div class="w-8 rounded-full bg-base-300">
                            <img
                                width="40"
                                height="40"
                                decoding="async"
                                src=avatar
                                alt=""
                                referrerpolicy="no-referrer"
                            />
                        </div>
                    </a>
                    <p class="min-w-0 flex-1 truncate text-sm">
                        <a class="font-semibold hover:underline" href=profile>
                            {heading}
                        </a>
                        <span class="opacity-70">{format!(" {said}")}</span>
                    </p>
                    <time class="text-xs opacity-50" datetime=created_at>{age}</time>
                </div>
                {extra.with_value(|extra| extra.as_ref().map(|extra| extra()))}
                {body.with_value(|body| body.as_ref().map(|body| body()))}
            </div>
        </li>
    }
}

/// How wide a screen's content column is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Width {
    /// Timelines and settings: the reading column.
    Standard,
    /// Moderation and administration: tables of decisions.
    Wide,
    /// Sign-in and the OAuth callback: one card, nothing beside it.
    Narrow,
}

impl Width {
    #[must_use]
    pub const fn class(self) -> &'static str {
        match self {
            Self::Standard => "max-w-2xl",
            Self::Wide => "max-w-3xl",
            Self::Narrow => "max-w-md",
        }
    }
}

/// A screen's content column.
///
/// One wrapper for every screen on both paints. The padding belongs to the frame, not to
/// the screen, so a page that adds its own `px-4` is a page whose column is four units
/// narrower than the same page painted by the server — which is the width jump this
/// component exists to remove.
#[component]
pub fn Screen(
    #[prop(default = Width::Standard)] width: Width,
    children: Children,
) -> impl IntoView {
    view! {
        <section class=format!("mx-auto flex w-full flex-col gap-3 px-2 {}", width.class())>
            {children()}
        </section>
    }
}
