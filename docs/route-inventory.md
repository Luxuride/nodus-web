# Route inventory: the client's routes, the shared pages, and the one dispatcher

Snapshot taken 2026-11-09 ~20:10 against a **dirty, moving working tree** (HEAD `2d0c83b`).
67 files are modified and `crates/view/src/route/screens/` gained four screens at 20:04–20:07
while this inventory was being read. Anything marked *dispatched* was already wired through
`RouteView` at that moment; anything marked *client-only* still renders from
`crates/web-client/src/routes`. Re-check `crates/view/src/route/mod.rs` before acting.

Read this with `docs/13-unification-plan.md` (interaction rule, "match the client", Err is a
state) and `docs/14-ssr-in-web.md` (crate roles, feature gating).

## 0. The seam as it stands

`crates/view/src/route/mod.rs` (362 lines) is the whole contract:

| item | line | shape |
| --- | --- | --- |
| `TimelineKind` | 29 | `Home`, `Local`, `Federated` |
| `TimelineReply` | 37 | `{ items: Vec<Status>, cursor: Option<String>, held: bool, unavailable: bool }` + `unavailable()` |
| `SessionView` | 57 | `{ host, account_id, acct }` |
| `Action` | 65 | 20 variants (favourite, boost, bookmark, delete, vote, follow, mute, block, answer request, conversation read/mute, list create/rename/delete/member, filter delete, tag feature/unfeature, scheduled cancel, profile update) |
| `ActionResult` | 90 | `None | Status | Conversation | Relationship | List | Filter | Account` |
| `trait Runtime` | 101 | `revision`, `timeline`, `load_timeline`, `page`, `load_page`, `status`, `context`, `account`, `relationship`, `load_thread`, `load_profile`, `session`, `follow_requests`, `load_follow_requests`, `lists`, `load_lists`, `list_accounts`, `load_list_accounts`, `action`, `now_millis`, `navigate` |
| `noop<In>()` | 218 | the stateless paint's callback: present, inert |
| `RouteView` | 227 | `(runtime, route, card: CardRenderer, header: Option<ChildrenFn>, banner: Option<ChildrenFn>)` |
| `TimelineScreen` | 282 | the one screen the trait was designed around |

Reads are synchronous snapshots (`Runtime::timeline`, `Runtime::page`, `Runtime::status`, …); a
`None` means "not held", and the screen loads through a `load_*` method whose completion arrives
as `Callback<Result<…, String>>`. Defaults on the trait are the stateless answer: `page` → `None`,
`load_page` → `Err("This page is not available without the app.")`, `action` → `Err("This action
is not available without the app.")`, `navigate` → no-op.

`RouteView` dispatches today:

| route | line | screen | state |
| --- | --- | --- | --- |
| `Home | Local | Federated` | 235 | `TimelineScreen` → `pages::timeline::TimelinePage` | shared |
| `Notifications` | 246 | `screens::notifications::NotificationsScreen` | shared |
| `Messages` | 250 | `screens::messages::MessagesScreen` | shared |
| `Saved(shelf)` | 254 | `screens::saved::SavedScreen` | shared |
| `Directory` | 258 | `screens::directory::DirectoryScreen` | shared |
| `Scheduled` | 262 | `screens::scheduled::ScheduledScreen` | shared |
| everything else | 271 | `Screen` + `PageHeader "Not available"` + `Note` | placeholder |

So 6 of 40 client routes are shared; 34 still come from `crates/web-client/src/app.rs` (40 `Route::`
arms, unchanged since 18:39).

Runtimes:

- **browser** — `crates/web-client/src/state/runtime.rs:18` `pub fn runtime() -> Arc<dyn Runtime>`,
  `struct ClientRuntime` (:23). Reads from the SDK store through `bridge::{pages, timeline, actions}`;
  loads `spawn_local`; `action` maps every variant to a `bridge::actions::*` call and re-dispatches
  `Command::StatusesUpserted` for a `Status` reply; `navigate` → `crate::router::navigate(Route::from_path(&path))`.
  `follow_requests()` (:120) and `lists()` (:127) return `None` by design — those reads are load-only.
- **server** — `crates/web-server/src/runtime.rs:12` `pub struct StaticRuntime { statuses, cursor }`,
  `StaticRuntime::page(statuses, cursor) -> Arc<dyn Runtime>` (:20). It answers `timeline` only
  (`held: true`), `load_timeline` → `Ok(())`, `now_millis` → `rows::now_millis()`; every other read
  falls to the trait default, i.e. "unavailable".

The server therefore does **not** call `RouteView` except through `TimelineScreen`. Its route work
lives in `crates/web-server/src/main.rs` (router ~:130) → `pages.rs` → `document.rs`:

| URL | server path |
| --- | --- |
| `/` | `main.rs:247` `index` → `document::home` (:206) if signed in, else `document::landing` (:382) |
| `/home` | `main.rs:311` `home` → `document::home` |
| `/local`, `/federated` | `main.rs:64` `public_feed` → `document::feed` (:214) |
| `/notifications` | `pages.rs:57` `ScreenKind::Notifications` → `NotificationsPageProps::server` |
| `/messages` | `pages.rs:69` → `MessagesPageProps::server` |
| `/saved/{shelf}` | `pages.rs:81` → `read_saved` (:158) → `SavedPageProps::server` |
| `/lists` | `pages.rs:82` → `ListsPageProps::server` (lists only, no panels) |
| `/scheduled` | `pages.rs:94` → `ScheduledPageProps::server` |
| `/directory` | `pages.rs:106` → `DirectoryPageProps::server` |
| `/search` | `pages.rs:118` → `read_search` (:198) → `SearchPageProps::server` |
| `/accounts/{id}` | `pages.rs:134` → `document::profile_page` (:267) |
| `/status/{id}` | `pages.rs:148` → `document::thread_page` (:321) |
| everything else | `main.rs:264` `shell` → `document::shell` (:395): `SignInPage` (:402) or `ShellPage` (:429) |

`document::page_of` (:365) is the seam where a rendered tree becomes a document; `pages.rs:38`
`painted()` is the `Owner::new().with(..)` wrapper a page must be built in (a `For` inside a body
asks `Owner::current()` and panics "no reactive owner" otherwise).

The server's prop shortcuts live in `crates/view/src/components/props/mod.rs` (184 lines):
`held<T>(T) -> Signal<T>` (:29), `none<T>() -> Signal<Option<T>>` (:34), `settled() -> Signal<bool>`
(:39, always `false`), `ages(now_millis) -> Callback<String, String>` (:44), and one
`XPageProps::server(..)` per shared page: `MessagesPageProps::server` (:51), `NotificationsPageProps::server`
(:74), `ListsPageProps::server` (:97), `ScheduledPageProps::server` (:116), `DirectoryPageProps::server`
(:130), `SavedPageProps::server` (:147), `SearchPageProps::server` (:164). Each fills the data and writes
`None` for every verb — that is the "server has no callbacks" rule enforced at one place.

## 1. Matrix

Legend per entry: **fn** = exported signatures with line; **page** = the shared component rendered;
**reads** = bridge/state/route inputs; **writes** = mutations or navigation; **slots** = platform-only
`ChildrenFn`/`AnyView`/`CardRenderer` props; **server** = exact counterpart or `none/skeleton`;
**notes** = migration hazards. Route files not listed here do not exist.

### 1.1 `routes/mod.rs`

- **fn** — module tree only: `account, admin (cfg feature="admin"), callback, compose, conversations,
  directory, feed, filtering, format, home, lists, moderation, notifications, people, saved, scheduled,
  search, settings, sign_in, status`.
- **page** — none (re-export surface; `pub use home::{Home, PrimaryTabs, TimelineHeader}`,
  `pub use status::{PublicThread, StatusCard, Thread}`).
- **server** — n/a.

### 1.2 `routes/account/` (3 files)

**`account/mod.rs`** (324 lines)
- **fn** — `pub(super) enum Verb { Follow, Mute, Block }` :61; `pub fn AccountView(id: String) -> impl IntoView` :73;
  `pub(super) fn toggle(…)` :295 (fans out at :314–316).
- **page** — `nodus_view::pages::accounts::ProfilePage` (`pages/accounts/[id]/mod.rs:32`):
  `account: Account, relationship: Option<Relationship>, statuses: Signal<Vec<Status>>, show_statuses: bool, tabs: Option<ChildrenFn>, after: Option<ChildrenFn>, card: Option<CardRenderer>, age_of: Option<Callback<String,String>>, on_follow/on_mute/on_block: Option<Callback<(String,bool)>>, on_avatar: Option<Callback<String>>`.
- **reads** — `bridge::{actions, revision, timeline}` :21–23; account timeline snapshot
  `timeline::read(which.get_value(), None)` :82; `timeline::ensure` :115/:231; `actions::account` :214;
  `actions::relationship` :222.
- **writes** — `actions::follow/mute/block` :314–316; router navigation for followers/following and
  compose; `nodus_sdk::Command` dispatch via `dispatch`.
- **slots** — `tabs` = `AccountTabs(tab, set_tab)` (`account/tabs/mod.rs:15`); `after` = the
  followers/following list; `card` = `account/profile/mod.rs:16` `pub(super) fn card() -> CardRenderer`
  (delegates to `routes::status::StatusCard`); `on_avatar` → media lightbox.
- **server** — `pages.rs:134` `/accounts/{id}` → `client.account` + `client.account_statuses(PAGE)` →
  `document::profile_page` (:267). Followers/following are **not** painted (no `ScreenKind` for them;
  the client's `Route::Account` also carries no tab).
- **notes** — unique type: `Relationship` decides the follow/mute/block verbs; `show_statuses` is a
  client-only tab fact; `Verb::toggle` needs a per-row optimistic update tied to a `Status`/`Relationship`
  reply. `age()` (`account/profile/mod.rs:21`) is a clock read that must become `age_of`.

**`account/profile/mod.rs`** — `pub(super) fn card() -> CardRenderer` :16; `pub(super) fn age(created_at: &str) -> String` :21. Helper only, no route body.

**`account/tabs/mod.rs`** — `pub fn AccountTabs(tab: ReadSignal<Tab>, set_tab: WriteSignal<Tab>)` :15. Pure view over a client-local `Tab`.

### 1.3 `routes/admin/` (4 files, `feature = "admin"`)

**`admin/mod.rs`** (92 lines)
- **fn** — `pub fn Admin(section: String) -> impl IntoView` :36; `fn standing_of(staff: admin::Staff) -> StaffStanding` :86.
- **page** — `pages::admin::AdminPage` (`pages/admin/mod.rs:105`): `active: String, standing: Signal<Option<StaffStanding>>, on_grant: Option<Callback<()>>, on_navigate: Option<Callback<String>>, children: ChildrenFn`; `Section` = Overview/Domains/Announcements.
- **reads** — `bridge::admin::staff()` :45, gated `signal(None::<admin::Staff>)`.
- **writes** — grant = `routes::settings::admin::request_grant()` :61; `router::navigate(Route::Admin(slug))` :74.
- **slots** — `children` = eager `<Show>`-less match over the three sections (each mounts only when current).
- **server** — none. `document::staff_doors()` (`document.rs:126`) only adds rail entries; no `ScreenKind::Admin`.
- **notes** — `StaffStanding` (view) vs `admin::Staff` (client) is a type bridge in `standing_of`; the
  grant path opens a browser popup (`oauth::begin_admin`), so `on_grant` must stay a client callback.

**`admin/overview/mod.rs`** — `pub fn Overview()` :19; `const DAYS: f64 = 7.0`; reads `admin::measures(&keys, DAYS)` :29 with `MEASURES`; page `pages::admin::overview::OverviewPage` (`measures: Signal<Vec<AdminMeasure>>, error: Signal<Option<String>>`); server none.
**`admin/domains/mod.rs`** — `pub fn Domains()` :21; `admin::domain_allows()` :32, `create_domain_allow` :47, `delete_domain_allow` :60; page `DomainsPage(items, domain, busy, error, on_domain, on_allow, on_withdraw)`; server none.
**`admin/announcements/mod.rs`** — `pub fn Announcements()` :20; read-only: `notices::announcements()` :17; page `AnnouncementsPage(items, error)`; server none. Comment in file: there is no admin API to create/withdraw an announcement.

### 1.4 `routes/callback/mod.rs` (1 file, 76 lines)

- **fn** — `pub fn Callback() -> impl IntoView` :16; private `fn strip_query()`.
- **page** — `nodus_view::pages::signing_in::SigningIn` :55 (`error`, `back`, `on_back`).
- **reads** — `window.location().search()` :22 → `oauth::finish(&search).await`; `oauth::Finished { purpose, host, token }`.
- **writes** — `bridge::keep_admin_token` :33 then `navigate(Route::Settings(SECTION_DEFAULT))` :35;
  else `bridge::sign_in(&host, &token)` :44 then `navigate(Route::Feed)`; `history.replace_state_with_url`
  :73 to strip the query.
- **slots** — none.
- **server** — none (OAuth PKCE is browser-only; the server's `/api/session` POST is the sign-in boundary, `main.rs:183`).
- **notes** — browser-only by nature: location, history, token exchange. Keep as a client route that
  dispatches a shared `SigningIn` screen; `Want` must never need it.

### 1.5 `routes/compose/` (13 files)

**`compose/mod.rs`** — `pub fn Compose()` :22; `pub fn ComposeEdit(id: String)` :32 (`actions::status(&id)` :42,
fallback `pages::compose::ComposeLoading` :10); `pub(crate) use form::ComposeForm; pub mod inline;`.
**`compose/form/mod.rs`** (211 lines) — `pub fn ComposeForm(initial: Option<Status>)` :55; builds
`text/visibility/spoiler/language/warning_open/sensitive/media/poll` signals + `editing: StoredValue<Option<String>>`;
reads `notices::preferences()` :38; writes `actions::edit`/`actions::publish` :100–101; navigates to
`Route::Thread{host: remembered_host(), id}` or `Route::Home` :112–114; renders
`pages::compose::form::ComposeForm` (`pages/compose/form/mod.rs:17`: `label: String, body: ChildrenFn,
children: ChildrenFn, error: Option<Signal<Option<String>>>`) whose `Body` is a `FieldBox` around
`crate::ui::editor::RichEditor`.
**`compose/form/buttons/mod.rs`** — `pub fn ComposerActions(text: ReadSignal<String>, busy: ReadSignal<bool>,
editing: StoredValue<Option<String>>, on_submit: Callback<()>)` :14; cancel → `Route::Home` :31.
**`compose/form/initial/mod.rs`** — `pub struct Initial { editing: Option<String>, text: String, visibility: Visibility,
spoiler: String, warning_open: bool, sensitive: bool }` :6; `Initial::of(Option<&Status>)` uses
`plain_text(&status.content)`.
**`compose/form/options/mod.rs`** — `pub fn ComposerOptions(visibility, set_visibility, warning_open,
set_warning_open, sensitive, set_sensitive, spoiler, set_spoiler, language, set_language)` :18; renders
`pages::compose::form::options::ComposerOptions` (`Signal` props + `on_visibility/on_toggle_warning/on_sensitive/on_spoiler`
callbacks + `language: Option<ChildrenFn>`).
**`compose/attachments/mod.rs`** — `pub fn request_pick()` :38 (a DOM `<input type=file>` from
`window.document()` :40); `pub fn Attachments(media: RwSignal<Vec<MediaAttachment>>, set_error: WriteSignal<Option<String>>)` :122;
`actions::upload_media_with_progress` :101, `actions::update_media` :156.
**`compose/inline/mod.rs`** (542) — `pub fn InlineComposer(which: StoredValue<nodus_sdk::Timeline>, set_reload: WriteSignal<u32>)` :58;
`actions::schedule` :436 / `actions::publish` :439; `timeline::refresh` :494; `nodus_view::pages::compose::inline::poll::DURATIONS`
:317; `EmojiHints` :532; renders `pages::compose::inline::InlineComposer` (`body: ChildrenFn, controls: ChildrenFn,
error/notice/done`).
**`compose/inline/bar/mod.rs`** — `pub(super) struct Notice` :25; `pub(super) fn ComposerBar(text, visibility,
set_visibility, at, set_at, busy, on_add_media: Callback<()>, emoji_open, set_emoji_open, on_add_poll: Callback<()>,
language, set_language, on_submit: Callback<()>)` :36 → `pages::compose::inline::bar::ComposerBar` (`Signal` props,
`limit`, `schedule_open`, optional callbacks, `language/emoji_picker/schedule_panel: Option<ChildrenFn>`).
**`compose/inline/schedule/mod.rs`** — `pub(super) fn SchedulePanel(at: ReadSignal<String>, set_at: WriteSignal<String>, open: RwSignal<bool>)` :19;
re-exports `split`; renders `pages::compose::inline::schedule::SchedulePanel` with the clock, `QuickTime`s, month
label and grid cells computed here (client clock).
**`compose/inline/drafts_panel/mod.rs`** — `pub(super) fn DraftsPanel(open: Signal<bool>, insert: Callback<usize>)` :20; localStorage-backed drafts.
**`compose/inline/emoji/mod.rs`** — `pub use …::Suggestions` :22; `pub fn pending(text: &str) -> Option<(usize, String)>` :30;
`pub fn catalogue() -> ReadSignal<Vec<CustomEmoji>>` :48 (`actions::custom_emojis` :51); `pub fn complete(text, colon, shortcode) -> String` :61.
**`compose/inline/emoji_toggle/mod.rs`** — `pub(super) fn EmojiToggle(open, set_open, busy)` :24; a document click listener (:70).
**`compose/inline/poll/mod.rs`** — `pub(super) fn as_draft(…)` :18; re-export `PollEditor` :43; `pub(super) fn focus_option(index)` :49 (DOM).
- **page (all)** — `pages::compose::…` (form, inline, attachments, poll, schedule, drafts, hints).
- **server** — none: there is no `ScreenKind::Compose`; `/compose` gets `document::shell` (skeleton).
- **notes** — the heaviest group: ProseMirror mount, file input, XHR upload progress, localStorage drafts,
  emoji catalogue, client clock for the schedule grid. **Keep the page body client-supplied** through the
  existing `body`/`children`/`controls` `ChildrenFn` slots; the dispatcher only needs to render the frame
  (`ComposeLoading` for `/compose/{id}` until the status arrives).

### 1.6 `routes/conversations/mod.rs` (1 file, 137 lines)

- **fn** — `pub fn Conversations() -> impl IntoView` :76.
- **page** — `pages::messages::MessagesPage` (`pages/messages/mod.rs:27`): `conversations, muted: Signal<Vec<String>>,
  loading, error, cursor, age_of: Option<Callback<String,String>>, on_open/on_read/on_mute/on_more`.
- **reads** — `pages::held` :79, `pages::items(&PageKind::Conversations, …)` :87, `pages::ensure_after` :98.
- **writes** — `actions::mute_conversation` :33, `actions::mark_conversation_read` :53, `dispatch`, router open.
- **slots** — none (the page draws its own `ConversationRow`).
- **server** — `pages.rs:69` → `MessagesPageProps::server(conversations, ages)` (**but** `SavedScreen`/`MessagesScreen`
  already exist in view, so this URL is now dispatched twice: `RouteView` for the client and `pages.rs` for the server).
- **notes** — `muted` is client-local (the payload has no muted flag); `on_open` both navigates and marks read.

### 1.7 `routes/directory/mod.rs` (1 file, 60 lines)

- **fn** — `pub fn Directory() -> impl IntoView` :22.
- **page** — `pages::directory::DirectoryPage` (`pages/directory/mod.rs:22`): `local_only, accounts, loading, error,
  cursor, on_scope: Option<Callback<bool>>, on_open: Option<Callback<String>>, on_more`.
- **reads** — `pages::accounts(&kind())` :34, `pages::ensure_after(kind, after)` :42, `revision`.
- **writes** — `router::navigate(Route::Account(who))` :61 (and the scope flip re-reads).
- **server** — `pages.rs:106` → `DirectoryPageProps::server` (always `local_only = true`).
- **notes** — directory pages by **offset** (`PageKind::Directory(bool)` cursor is a number).

### 1.8 `routes/feed/` (9 files)

**`feed/mod.rs`** — `pub fn Feed()` :26 → `pages::feed::FeedPage` (`pages/feed/mod.rs:39`: `tab: Signal<FeedTab>,
on_tab, header: Option<ChildrenFn>, people: ChildrenFn, posts: ChildrenFn`), sections `routes/feed/{people,posts}`.
**`feed/people/mod.rs`** — `pub fn People()` :21 → `pages::feed::accounts::PeopleList`; reads `feed_ui`, `revision`,
`with_sdk(|sdk| sdk.state().feed.get())` :27 (degraded :35, next_cursor :39); writes `reload_feed`, `navigate(Route::Settings(..))` :59.
**`feed/posts/mod.rs`** — `pub fn Posts()` :24 → `pages::feed::posts::PostList`; reads `posts_ui`, `revision`,
`sdk.state().posts` :32/:36; writes `load_posts_once`, `reload_posts`.
**`feed/posts/card/mod.rs`** — `pub fn PostCard(item: PostItem)` :16 → `pages::feed::posts::PostCard`; impressions
`begin_impression`/`finish_impression` :11.
**`feed/posts/text/mod.rs`** — `pub fn plain_text(html: &str) -> String` :15 (helper used by compose/initial, banner, filtering).
**`feed/item/mod.rs`** — `pub(super) fn FeedItem(item: AccountItem)` :21 → `pages::feed::accounts::AccountCard` :87; impressions; `Toast`.
**`feed/actions/mod.rs`** — `pub(super) struct ItemActions` :19; `pub(super) fn item_actions(…)` :36; `follow_item`,
`unfollow_item`, `not_interested`, `with_sdk` :14; session host :105 (for follow).
**`feed/reason/mod.rs`** — text helpers: `reason_line` :17, `reason_title` :33, `sample_followers_line` :49,
`similar_line` :83, `follower_label` :91, `degraded_line` :102.
**`feed/share/mod.rs`** — `pub(super) struct ShareReport` :14; `copy_post_link` :35 (`{origin}/s/{host}/{id}`);
`write_clipboard` :53.
- **server** — none. `/feed` gets `document::shell`. (Careful: `document::feed` is the **/local and /federated**
  builder, not this route.)
- **notes** — the whole feed reads SDK feed/posts state, not the page store, and has impression side-effects;
  impressions must stay in the client (view may not do I/O).

### 1.9 `routes/filtering/mod.rs` and `routes/format/mod.rs`

- `filtering` — `pub fn judge(status: &Status, context: FilterContext) -> Verdict` :29 (reads
  `pages::items(&PageKind::Filters, …)` :30); `#[component] pub fn Filtered(status: Status, context: FilterContext)` :51 —
  the `CardRenderer` wrapper every status list uses. Not a route.
- `format` — `pub fn relative_time(rfc3339: &str) -> String` :11. Not a route; the clock read the shared pages
  deliberately do not make.

### 1.10 `routes/home/` (4 files)

**`home/mod.rs`** — `pub fn Home(which: Timeline)` :23. The only route already written as a dispatcher call:
builds `Runtime` from `state::runtime::runtime()`, `card` = `CardRenderer` over `routes::status::StatusCard`,
`header` = `TimelineHeader`, `banner` = `Banner`, and renders `<RouteView runtime route card header banner/>`.
**`home/header/mod.rs`** — `pub fn TimelineHeader()` :16.
**`home/tabs/mod.rs`** — `pub fn PrimaryTabs()` :15 (reads `router::route()` :16, navigates by path :40).
**`home/banner/mod.rs`** — `pub fn Banner()` :18; `notices` :14; `pages::timeline::banner::TimelineBanner` :53 with
`Announcement` :57.
- **server** — `document::home` (:206) for `/` and `/home`, `document::feed` (:214) for `/local`+`/federated`;
  the header is `document::timeline_header` (:190) → `TimelineHeaderPlaceholder`; the banner is absent.
- **notes** — this is the template to copy: the route keeps only slot construction.

### 1.11 `routes/lists/` (5 files)

**`lists/mod.rs`** (141)
- **fn** — `pub fn Lists() -> impl IntoView` :32.
- **page** — `pages::lists::ListsPage` (`pages/lists/mod.rs:31`): `lists, selected: Signal<Option<String>>, title, busy, error,
  on_title, on_create, row: Option<Callback<List, AnyView>>, members: Option<ChildrenFn>, timeline: Option<ChildrenFn>`.
- **reads** — `bridge::seed::lists()` :36 (server-painted island, adopted once), `pages`?? no — `actions::lists()` :53.
- **writes** — `actions::create_list(name)` :87; `actions::rename_list`/`delete_list` in the row; `timeline::ensure` in the panel.
- **slots** — `row` = `ListRow` (**client-only** controls: rename input, delete), `members` = `Members`, `timeline` = `ListTimeline`.
- **server** — `pages.rs:82` → `ListsPageProps::server(lists)` — lists only, no row/member/timeline panels.
- **notes** — `seed::lists()` is an SSR island read (`bridge/seed.rs:151`); the server's `ListsPageProps::server`
  passes `selection = none()`, so a server paint shows the picker but never opens a list.

**`lists/row/mod.rs`** — `pub fn ListRow(list: List, selected: Signal<Option<String>>, set_selected: WriteSignal<Option<String>>,
busy: Signal<bool>, set_error: WriteSignal<Option<String>>, reload: Callback<()>)` :20 → `pages::lists::ListPickerRow`
(`list, selected: bool, on_select, controls: Option<ChildrenFn>`); `actions::rename_list` :49, `actions::delete_list` :64.
**`lists/timeline/mod.rs`** — `pub fn ListTimeline(list_id: Signal<Option<String>>)` :20 → `pages::lists::ListTimeline`
(`list_id, statuses, cursor, loading, error, on_more: Option<Callback<()>>, age_of, card: Option<Callback<Status,AnyView>>`);
`timeline::read(Timeline::List(id), None)` :31, `timeline::ensure` :44, `revision`.
**`lists/members/mod.rs`** — `pub fn Members(list_id: Signal<Option<String>>)` :22 → `pages::lists::Members`
(`list_id, members, busy, error, on_remove, add: Option<ChildrenFn>`); `actions::list_accounts` :55, `actions::set_list_member(false)` :38.
**`lists/members/add/mod.rs`** — `pub fn AddPeople(list_id: Signal<Option<String>>, set_error: WriteSignal<Option<String>>,
on_add: Callback<()>)` :18 → `pages::lists::AddPeople` (`query, candidates, busy, on_query, on_search, on_add`);
`actions::search(text, true)` :39, `actions::set_list_member(true)` :57.

### 1.12 `routes/moderation/` (6 files)

**`moderation/mod.rs`** (95) — `pub fn Moderation(section: String)` :37 → `pages::moderation::ModerationPage`
(`pages/moderation/mod.rs:125`: `active: String, standing: Signal<Option<StaffStanding>>, on_grant, on_navigate, children: ChildrenFn`);
`admin::staff()` :46, `standing_of` :89, grant :62, `router::navigate(Route::Moderation(slug))` :76; `Section` = Reports/Accounts/Registrations/Federation.
**`moderation/accounts/mod.rs`** — `pub fn AccountList()` :13 → `pages::moderation::accounts::AccountsPage`
(`accounts, local_only, busy, error, on_local_only: Option<Callback<bool>>, on_moderate: Option<Callback<(String, ModerationAction)>>`);
`admin::accounts(local_only)` :22, `admin::moderate` :46.
**`moderation/reports/mod.rs`** — `pub fn ReportQueue()` :14 → `ReportsPage(reports, busy, error, age_of, on_resolve)`;
`admin::reports(ReportState::Open)` :22, `admin::resolve_report` :43.
**`moderation/registrations/mod.rs`** — `pub fn Registrations()` :20 → `RegistrationsPage(items, busy, error, on_refresh, on_decide: Option<Callback<(String,bool)>>)`;
`admin::pending_accounts` :28, `approve_account`/`reject_account` :48–50.
**`moderation/federation/mod.rs`** — `pub fn Federation()` :14 → `<BlockedList/>` only.
**`moderation/federation/blocked/mod.rs`** — `pub(super) fn BlockedList()` :13 → `FederationPage(blocks, domain, suspend,
busy, error, on_domain, on_suspend, on_block, on_remove)`; `admin::domain_blocks` :23, `create_domain_block(&name, severity)` :52
(`DomainSeverity::{Suspend, Silence}`), `delete_domain_block` :66.
- **server** — none; `/moderation/*` gets `document::shell`.
- **notes** — admin transport is client-only (see §3 blockers); `StaffStanding` is the shared gate type.

### 1.13 `routes/notifications/` (2 files)

**`notifications/mod.rs`** (137) — `pub fn Notifications()` :31 → `pages::notifications::NotificationsPage`
(`pages/notifications/mod.rs:31`: `notifications, cursor, loading, error, age_of, on_more, on_open,
status_card: Option<Callback<Status, AnyView>>, request_actions: Option<Callback<String, AnyView>>, banner: Option<ChildrenFn>`);
`pages::held/items/ensure/ensure_after(PageKind::Notifications)` :33–69; follow requests `actions::authorize_follow_request` :83
and `reject_follow_request` :87; `router::navigate(thread_route(&id))` :119; slots: `status_card` → `StatusCard`,
`request_actions` → `FollowRequestActions`, `banner` → `PendingRequests`.
**`notifications/pending/mod.rs`** — `pub fn PendingRequests(set_error: WriteSignal<Option<String>>)` :19 →
`pages::notifications::RequestBanner` (`requests, busy: Option<Signal<bool>>, on_answer: Option<Callback<(String,bool)>>, on_open`);
`actions::follow_requests()` :36; nav `Route::Account(id)` :67. Failures are silent by design.
- **server** — `pages.rs:57` → `NotificationsPageProps::server(items, ages, None)`; the banner and request actions are
  never painted (no `follow_requests` read on the server).
- **notes** — the shared screen now reads `runtime.follow_requests()` (browser returns `None`), so the banner is empty
  until a `Want::FollowRequests` exists.

### 1.14 `routes/people/mod.rs` (1 file)

- **fn** — `pub fn PeopleList(kind: Kind, id: String)` :33; `pub use nodus_view::pages::people::Kind` :19.
- **page** — `pages::people::PeoplePage` (re-export of `pages/moderation/[section]/mod.rs:59`): `kind: Kind, accounts,
  loading, error, cursor, on_open, on_undo: Option<Callback<String>>, on_more`; `Kind::{Followers, Following, Mutes, Blocks}`.
- **reads** — `pages::held` :43, `pages::accounts(&page(kind, &id))` :52, `pages::ensure_after` :61.
- **writes** — undo: `actions::mute(id, false)` :80 / `actions::block(id, false)` :81; open → `Route::Account` :75.
- **server** — none (`/accounts/{id}/followers` is not a `ScreenKind`; `document::profile_page` paints the profile only).
- **notes** — one screen, four URLs; the page kind is `PageKind::{Followers,Following,Mutes,Blocks}` and the undo verb
  comes from `Kind::undo()` (view), so this is the cheapest large win.

### 1.15 `routes/saved/mod.rs` (1 file)

- **fn** — `pub fn Saved(shelf: String)` :35.
- **page** — `pages::saved::SavedPage` (`pages/saved/[shelf]/mod.rs:28`): `shelf: Signal<Shelf>, items, loading, error,
  cursor, card: CardRenderer, on_shelf: Option<Callback<Shelf>>, on_more`.
- **reads** — `pages::held` :39, `pages::items(&kind(shelf))` :44, `pages::ensure_after` :57.
- **writes** — `router::navigate(Route::Saved(to.slug()))` :65.
- **server** — `pages.rs:81` → `read_saved` (:158) → `SavedPageProps::server(shelf, statuses, card)` (island kind `bookmarks`/`favourites`).
- **notes** — `Shelf::{Favourites, Bookmarks}` is shared; the shelf is also in the URL, so the screen must accept it.

### 1.16 `routes/scheduled/mod.rs` (1 file)

- **fn** — `pub fn Scheduled()` :21.
- **page** — `pages::scheduled::ScheduledPage` (`pages/scheduled/mod.rs:21`): `items, loading, error, age_of, on_cancel`.
- **reads** — `pages::held/items/ensure(PageKind::Scheduled)` :23–45.
- **writes** — `actions::cancel_scheduled` :59, `dispatch`.
- **server** — `pages.rs:94` → `ScheduledPageProps::server(items, ages)`.
- **notes** — trivial; the only oddity is the age format ("`{scheduled_at} · {relative}`").

### 1.17 `routes/search/mod.rs` (1 file)

- **fn** — `pub fn Search()` :29.
- **page** — `pages::search::SearchPage` (`pages/search/mod.rs:23`): `query, accounts, statuses, tags, searched, loading,
  error, on_query, on_search, age_of: Option<Callback<String,String>>, card: Option<Callback<Status, AnyView>>`.
- **reads** — `router::query_param("q")` :31 (the URL), `pages::read(&PageKind::Search(text))` :48, `pages::held` :64,
  `pages::ensure` :76.
- **writes** — router query update on submit.
- **server** — `pages.rs:118` → `read_search` (:198, `percent_decode` :244) → `SearchPageProps::server(query, results, ages, card)`.
- **notes** — **the route carries no query**; `Route::Search` is a unit variant while the page needs `q`. Both paints
  currently read the query out of band (client router / server URI). A `RouteView` that owns search must receive the
  query (see `Want::Page(PageKind::Search(q))` and the `query` prop proposal).

### 1.18 `routes/settings/` (17 files)

**`settings/mod.rs`** (109) — `pub fn Settings(section: String)` :51 → `pages::settings::page::SettingsPage`
(`pages/settings/page.rs:11`: `section: Signal<SettingsSection>, on_navigate: Option<Callback<SettingsSection>>, children: ChildrenFn`);
`with_sdk(|sdk| sdk.state().session.get()…)` :58/:68 for `host`/`acct`; `router::navigate(Route::Settings(next.slug()))` :79;
`SettingsSection::{Account, Profile, Nodus, Filters, Instance}` (`pages/settings/nav.rs:7`); body is a `Show` over the
current tab with sections Account, Profile, Nodus, Filters, Instance.
**`settings/accounts/mod.rs`** — `pub fn AccountsCard()` :16 → `pages::settings::account::AccountsCard` :57
(`accounts, busy, error, on_switch, on_sign_out`); `bridge::active_key()` :23, `bridge::account_list()` :24, `bridge::switch` :41,
`bridge::session::sign_out()` :51; `AccountItem{current, key, label, host}`.
**`settings/admin/mod.rs`** — `pub async fn request_grant() -> Result<(), String>` :? (imports `admin_host` :6): needs
`admin_host()` then `oauth::begin_admin(&host)`; error "There is no instance to ask yet."
**`settings/clock/mod.rs`** — `pub fn ClockCard()` :11 → `pages::settings::profile::ClockCard` (`selected: Signal<ClockStyle>,
sample, on_select`); `clock_format` signal`/`set`; `ClockFormat::{TwentyFour, Twelve}`.
**`settings/feeds/mod.rs`** — `pub fn FeedsCard()` :14 → `pages::settings::feeds::FeedsCard` (`feeds: Signal<Vec<FeedItem>>,
on_toggle: Option<Callback<(String,bool)>>`); `feeds::revision/enabled/set`, `Feed::ALL`, `feed.key()/label()/description()/optional()`.
**`settings/filters/mod.rs`** — `pub fn FiltersCard()` :23 → `pages::settings::lists::FiltersCard` (`filters, busy, error,
on_delete, add_form: ChildrenFn`); `pages::items(PageKind::Filters)` :30, `pages::refresh` :63, `pages::ensure` :75,
`actions::delete_filter` :86; `FilterItem{id,title,detail}`; `add_form` = `NewFilterForm`.
**`settings/filters/new/mod.rs`** — `pub fn NewFilterForm(on_created: Callback<()>, set_error: WriteSignal<Option<String>>)` :16 →
`pages::settings::form::FilterFields`; `FilterDraft::new(name).action(..).keywords(..)`; `actions::create_filter` :47.
**`settings/instance/mod.rs`** — `pub fn InstanceCard()` :16 → `pages::settings::instance_card::InstanceCard`
(`info: Signal<Option<InstanceInfo>>, failed: Signal<bool>`); `actions::instance_info()` :23 (GET `/api/v2/instance`).
**`settings/moderation/mod.rs`** — `pub(crate) fn ModerationLinks()` :13 → `pages::settings::account::ModerationLinks`
(`on_mutes, on_blocks`); navigates `Route::Mutes`/`Route::Blocks` :16–17.
**`settings/profile/mod.rs`** — `pub(crate) fn ProfileCard()` :21; `const NOTE_MAX_CHARS: usize = 500` :13; reads the account
through `with_sdk(|sdk| sdk.read_account(session.account_id))` :27; `actions::update_profile` :62; renders
`pages::settings::form::ProfileFields` inside `components::panel::Panel`.
**`settings/sensitive/mod.rs`** — `pub(super) fn SensitiveCard()` :14 → `pages::settings::profile::SensitiveCard`
(`automatic, on_toggle`); reads/writes `routes::status::sensitive::*` (**localStorage**).
**`settings/session/mod.rs`** — `pub(crate) fn SessionCard(host: impl Fn() -> String + Copy + Send + Sync + 'static,
acct: impl …)` :10 → `pages::settings::account::SessionCard` (`host, acct, on_sign_out`); `bridge::sign_out`.
**`settings/sync/mod.rs`** — `pub fn install()` :49 (must run in `main` before mount; the file documents the
"Tried to access a reactive value that has already been disposed" panic); thread-local
`static UI: SyncUi { confirming: RwSignal<Option<SyncIntent>>, busy: RwSignal<bool>, result: RwSignal<Option<String>> }`;
`pub fn SyncCard()` :64 → `Panel title="Background sync"` + `pages::settings::sync::SyncRows` (`state: Signal<Option<bool>>,
busy, result, on_ask`) + `ConfirmDialog` (`confirming, on_cancel, on_confirm`); `with_sdk(|sdk| sdk.state().sync…)` :72;
`bridge::load_sync_consent()`.
**`settings/sync/authorize/mod.rs`** — `pub(super) fn start(result, busy, alive)` :17; `bridge::authorize_sync()` :11;
`window.open_with_url_and_target(&url, "_blank")` :24.
**`settings/sync/decide/mod.rs`** — `pub(super) fn run(intent: SyncIntent, …)` :16; enable → `authorize::start`, disable →
`bridge::set_sync(false, false)` :13.
**`settings/tags/mod.rs`** — `pub fn FeaturedTagsCard()` :19 → `pages::settings::lists::FeaturedTagsCard` (`tags, busy, error,
on_delete, add_form`); `pages::items(PageKind::FeaturedTags)` :25, `ensure` :46, `refresh` :53, `actions::unfeature_tag` :64;
`TagItem{id,label}`.
**`settings/tags/add/mod.rs`** — `pub fn AddTag(busy, set_busy, set_error, on_added: Callback<()>)` :12 →
`pages::settings::form::FeaturedTagField`; `actions::feature_tag` :33.
- **server** — none; `/settings/*` gets `document::shell`.
- **notes** — settings is 7 independently-mounted cards, each with its own read and write; the page shell
  (`SettingsPage`) is shared but every card still fetches on its own. Two cards are **browser state**
  (`sensitive` → localStorage, `sync` → consent + popup) and cannot be `Want`s.

### 1.19 `routes/sign_in/mod.rs` (1 file, 90 lines)

- **fn** — `pub fn SignIn(#[prop(optional)] note: Option<&'static str>)` :23.
- **page** — `pages::sign_in::SignInPage` :75 (`note, host, token, authorizing, pasting, authorize_error, token_error,
  startup_error, on_host, on_token, on_authorize, on_token_submit`).
- **reads** — `feed_ui().error` as `startup_error` :14.
- **writes** — `oauth::begin(&host)` (full-page redirect), `bridge::sign_in(&host, &token)`.
- **server** — `document::shell` (:395) paints `SignInPage` with every verb `None` when there is no session (`document.rs:402`).
- **notes** — not a `Route`; `app.rs` renders it when signed out. The server already shares this page.

### 1.20 `routes/status/` (9 files)

**`status/mod.rs`** (167) — `pub(super) fn thread_route(id: &str) -> Route` :35 (uses
`bridge::session::remembered_host()`, **localStorage**); `pub fn StatusCard(status: Status)` :48 → the shared
`nodus_view::status_card::StatusCard` (boost unwrap via `booster`, `author_path`, `thread_path`, `on_navigate`,
`on_reply/on_boost/on_favourite/on_bookmark`, `before_verbs`/`after_verbs`, `body`, `extra`); private
`fn inside_control(event: &web_sys::MouseEvent) -> bool`; `router::link` :92; `pub use public::PublicThread` :16.
**`status/body/mod.rs`** — `pub fn StatusBody(shown: ReadSignal<Status>, set_shown: WriteSignal<Status>)` :20 →
`nodus_view::status_body::StatusBody` (`status, on_open_media, on_vote, revealed_init`); `media::expand`;
`actions::vote` :70; `sensitive::reveals_automatically()`.
**`status/footer/mod.rs`** — `pub(super) struct CardActions { on_reply, on_boost, on_favourite, on_bookmark: Callback<String>,
before_verbs: ChildrenFn, after_verbs: ChildrenFn }`; `pub(super) fn card_actions(shown, set_shown, busy, set_busy, set_error) -> CardActions` :49;
`actions::reblog/favourite/bookmark` :66–94; reply → `thread_route` :58.
**`status/menu/mod.rs`** — `pub fn StatusMenu(shown: ReadSignal<Status>, set_error: WriteSignal<Option<String>>)` :16 →
`nodus_view::status_menu::StatusMenu`; `actions::viewer_id()` :23; `window.confirm_with_message("Delete this status?")` :29;
`actions::delete_status` :35 → `Route::Home` :37; edit → `Route::ComposeEdit` :52.
**`status/public/mod.rs`** — `pub fn PublicThread(host: String, id: String) -> AnyView` :23; empty host → `HostForm`, else
`pages::status::public::PublicThreadPage(host, status, ancestors, replies, loading, error, age_of)`; `public::status` :62,
`public::context` :65 (**no session, public API**); `HostForm` posts to `router::navigate(Route::Thread{..})` :38.
**`status/reply/mod.rs`** — `pub(super) fn ReplyBox(parent: String, set_replies: WriteSignal<Vec<Status>>,
set_error: WriteSignal<Option<String>>)` :16 → `pages::reply_card::ReplyCard` (`text, busy, on_input, on_submit`);
`actions::publish(&NewStatus::reply(text, &parent))` :34.
**`status/sensitive/mod.rs`** — `const KEY: &str = "nodus.sensitive"` :11, `const SHOW: &str = "show"` :12;
`pub fn reveals_automatically() -> bool` :19, `pub fn set_automatic(on: bool)` :27 (**localStorage**).
**`status/share/mod.rs`** — `pub(super) fn ShareButton(shown, set_error)` :19 → `nodus_view::status_share::ShareButton`
(`copied, on_copy`); `with_sdk(… session.host)` :30; `feed::share::copy_post_link` (**clipboard**).
**`status/thread/mod.rs`** — `pub fn Thread(id: String)` :34 → `nodus_view::thread::ThreadPage`
(`pages/status/[id]/mod.rs:67`: `ancestors, focused, replies, card: Option<CardRenderer>, age_of, reply_slot: Option<ChildrenFn>`);
`actions::status` :46 + `timeline::upsert` :50, `actions::thread` :59 + `timeline::upsert` :63; `card` = `CardRenderer` → `Filtered`;
`reply_slot` = `ReplyBox` + error note; fallback `Screen` with an error note.
- **server** — `/status/{id}` → `pages.rs:148` → `client.status`+`client.context` → `document::thread_page` (:321), which passes
  `card: None`, `age_of: Some(clock)`, `reply_slot: None` and `StatusColumn`-free text. `/s/{host}/{id}` is served by the
  public bundle, not this route (`router` excludes `/s/`).
- **notes** — cards, menus, share and reveal are browser-side; `StatusCard` is already a shared component but its
  `CardRenderer` wiring, clipboard and media lightbox stay here. `thread_route` depends on `remembered_host()`.

## 2. The shared pages' props (what a dispatcher has to fill)

Every page follows the same split: **data as `Signal`**, **verbs as `Option<Callback<..>>`**,
**platform-only pieces as `Option<ChildrenFn>` / `Option<Callback<.., AnyView>>` / `CardRenderer`**.
`Option<Callback<..>>` props must be `#[prop(optional_no_strip)]` or `None` becomes an enabled button
(`docs/14-ssr-in-web.md`).

| page (file:line) | data | verbs | slots |
| --- | --- | --- | --- |
| `NotificationsPage` `crates/view/src/pages/notifications/mod.rs:31` | `notifications: Signal<Vec<Notification>>, cursor, loading, error` | `age_of, on_more, on_open` | `status_card: Option<Callback<Status, AnyView>>`, `request_actions: Option<Callback<String, AnyView>>`, `banner: Option<ChildrenFn>` |
| `MessagesPage` `crates/view/src/pages/messages/mod.rs:27` | `conversations, muted: Signal<Vec<String>>, loading, error, cursor` | `age_of, on_open, on_read, on_mute: Option<Callback<(String,bool)>>, on_more` | — |
| `SavedPage` `crates/view/src/pages/saved/[shelf]/mod.rs:28` | `shelf: Signal<Shelf>, items, loading, error, cursor` | `on_shelf: Option<Callback<Shelf>>, on_more` | `card: CardRenderer` (required) |
| `SearchPage` `crates/view/src/pages/search/mod.rs:23` | `query, accounts, statuses, tags, searched, loading, error` | `on_query, on_search, age_of` | `card: Option<Callback<Status, AnyView>>` |
| `DirectoryPage` `crates/view/src/pages/directory/mod.rs:22` | `local_only, accounts, loading, error, cursor` | `on_scope: Option<Callback<bool>>, on_open, on_more` | — |
| `PeoplePage` `crates/view/src/pages/moderation/[section]/mod.rs:59` | `kind: Kind, accounts, loading, error, cursor` | `on_open, on_undo: Option<Callback<String>>, on_more` | — |
| `ListsPage` `crates/view/src/pages/lists/mod.rs:31` | `lists, selected, title, busy, error` | `on_title, on_create` | `row: Option<Callback<List, AnyView>>`, `members: Option<ChildrenFn>`, `timeline: Option<ChildrenFn>` |
| `ListPickerRow` `crates/view/src/pages/lists/row/mod.rs:17` | `list, selected: bool` | `on_select` | `controls: Option<ChildrenFn>` |
| `ListTimeline` `crates/view/src/pages/lists/timeline/mod.rs:19` | `list_id, statuses, cursor, loading, error` | `on_more: Option<Callback<()>>, age_of` | `card: Option<Callback<Status, AnyView>>` |
| `Members` `crates/view/src/pages/lists/members/mod.rs:14` | `list_id, members, busy, error` | `on_remove` | `add: Option<ChildrenFn>` |
| `AddPeople` `crates/view/src/pages/lists/add/mod.rs:14` | `query, candidates, busy` | `on_query, on_search, on_add` | — |
| `ScheduledPage` `crates/view/src/pages/scheduled/mod.rs:21` | `items, loading, error` | `age_of, on_cancel` | — |
| `ProfilePage` `crates/view/src/pages/accounts/[id]/mod.rs:32` | `account: Account, relationship: Option<Relationship>, statuses, show_statuses: bool` | `on_follow/on_mute/on_block: Option<Callback<(String,bool)>>, on_avatar, age_of` | `tabs: Option<ChildrenFn>`, `after: Option<ChildrenFn>`, `card: Option<CardRenderer>` |
| `ThreadPage` `crates/view/src/pages/status/[id]/mod.rs:67` | `ancestors: Vec<Status>, focused: Status, replies` | `age_of` | `card: Option<CardRenderer>`, `reply_slot: Option<ChildrenFn>` |
| `TimelinePage` `crates/view/src/pages/timeline/mod.rs:22` | `title, description, empty_note, error, loading, items, more` | `on_more: Option<Callback<()>>` | `header: Option<ChildrenFn>`, `banner: Option<ChildrenFn>`, `card: CardRenderer` |
| `ComposeForm` `crates/view/src/pages/compose/form/mod.rs:17` | `label: String` | `error: Option<Signal<Option<String>>>` | `body: ChildrenFn`, `children: ChildrenFn` |
| `SettingsPage` `crates/view/src/pages/settings/page.rs:11` | `section: Signal<SettingsSection>` | `on_navigate: Option<Callback<SettingsSection>>` | `children: ChildrenFn` |
| `SessionCard` / `AccountsCard` `crates/view/src/pages/settings/account.rs:22` / `:57` | `host, acct` / `accounts, busy, error` | `on_sign_out` / `on_switch, on_sign_out` | — |
| `FeedsCard` `crates/view/src/pages/settings/feeds.rs:22` | `feeds: Signal<Vec<FeedItem>>` | `on_toggle: Option<Callback<(String,bool)>>` | — |
| `FiltersCard` / `FeaturedTagsCard` `crates/view/src/pages/settings/lists.rs:20` / `:86` | `filters`/`tags`, `busy, error` | `on_delete` | `add_form: ChildrenFn` (required) |
| `ClockCard` / `SensitiveCard` `crates/view/src/pages/settings/profile.rs:19` / `:66` | `selected: Signal<ClockStyle>, sample` / `automatic` | `on_select` / `on_toggle` | — |
| `InstanceCard` `crates/view/src/pages/settings/instance_card.rs:12` | `info: Signal<Option<InstanceInfo>>, failed` | — | — |
| `ProfileFields` `crates/view/src/pages/settings/form/mod.rs:17` | `display_name, note, max_note` | `on_display_name, on_note, on_save` | — |
| `SyncRows` / `ConfirmDialog` `crates/view/src/pages/settings/sync/mod.rs:75` / `:154` | `state: Signal<Option<bool>>, busy, result` / `confirming: Signal<Option<SyncIntent>>` | `on_ask` / `on_cancel, on_confirm` | — |
| `ModerationPage` `crates/view/src/pages/moderation/mod.rs:125` | `active: String, standing: Signal<Option<StaffStanding>>` | `on_grant, on_navigate` | `children: ChildrenFn` |
| `AccountsPage` (moderation) `crates/view/src/pages/moderation/accounts/mod.rs:12` | `accounts: Signal<Vec<AdminAccount>>, local_only, busy, error` | `on_local_only: Option<Callback<bool>>, on_moderate: Option<Callback<(String, ModerationAction)>>` | — |
| `ReportsPage` `crates/view/src/pages/moderation/reports/mod.rs:13` | `reports, busy, error` | `age_of, on_resolve` | — |
| `RegistrationsPage` `crates/view/src/pages/moderation/registrations/mod.rs:13` | `items, busy, error` | `on_refresh, on_decide: Option<Callback<(String,bool)>>` | — |
| `FederationPage` `crates/view/src/pages/moderation/federation/mod.rs:16` | `blocks, domain, suspend, busy, error` | `on_domain, on_suspend, on_block, on_remove` | — |
| `AdminPage` `crates/view/src/pages/admin/mod.rs:105` | `active: String, standing: Signal<Option<StaffStanding>>` | `on_grant, on_navigate` | `children: ChildrenFn` |
| `OverviewPage` / `DomainsPage` / `AnnouncementsPage` | `measures, error` / `items, domain, busy, error` / `items, error` | — / `on_domain, on_allow, on_withdraw` / — | — |
| `FeedPage` `crates/view/src/pages/feed/mod.rs:39` | `tab: Signal<FeedTab>` | `on_tab: Option<Callback<FeedTab>>` | `header: Option<ChildrenFn>`, `people: ChildrenFn`, `posts: ChildrenFn` |
| `ReplyCard` `crates/view/src/pages/status/reply_card/mod.rs:16` | `text: String, busy: bool` | `on_input, on_submit` | — |
| `RequestBanner` / `FollowRequestActions` `crates/view/src/pages/notifications/banner/mod.rs:23` / `request/mod.rs:20` | `requests` / `account_id` | `on_answer: Option<Callback<(String,bool)>>, on_open` / `on_answer` | — |

Server-only constructors (all in `crates/view/src/components/props/mod.rs`): `held` :29, `none` :34,
`settled` :39, `ages` :44, then `MessagesPageProps::server` :51, `NotificationsPageProps::server` :74,
`ListsPageProps::server` :97, `ScheduledPageProps::server` :116, `DirectoryPageProps::server` :130,
`SavedPageProps::server` :147, `SearchPageProps::server` :164. There is **no** `::server` for
`ProfilePage`, `ThreadPage`, `PeoplePage`, `SettingsPage`, `ModerationPage`, `AdminPage`, `ComposeForm`,
`FeedPage` — those are assembled field-by-field at `crates/web-server/src/pages.rs:134/148` or
`document.rs:267/321`.

## 3. Proposed grouping: ten screens, one `Want`/`Reply`/`Action`

The existing design already says what to do: one `Runtime` method per *kind of read* couples every
screen to every platform. Replace the read half of the trait with **one `Want` in, one `Reply` out**
(and `want()` to start the read), keep `Action` for writes, and fold `ActionResult` into `Reply`.
New file: `crates/view/src/route/want.rs`; edits: `crates/view/src/route/mod.rs:65-215`.

```rust
// crates/view/src/route/want.rs
pub enum Want {
    Timeline(TimelineKind),                 // Home, Local, Federated, List(id), Account(id), Tag(..)
    Page(nodus_state::page::PageKind),      // every paged list, keyed exactly as the store keys it
    Status(String),
    Context(String),
    Account(String),
    Relationship(String),
    FollowRequests,
    Lists,
    ListAccounts(String),
    Session,
    InstanceInfo,
    Feed(FeedWant),                         // People { cursor }, Posts { cursor }
    Admin(AdminWant),                       // Staff, Reports(ReportState), Accounts{local}, Registrations,
                                            // DomainBlocks, DomainAllows, Measures{keys,days}, Announcements
}

pub enum Reply {
    Timeline(TimelineReply),
    Page(nodus_state::page::PageEntry),
    Status(Status), Context(Context), Account(Account), Relationship(Option<Relationship>),
    FollowRequests(Vec<Account>), Lists(Vec<List>), ListAccounts(Vec<Account>),
    Session(Option<SessionView>), InstanceInfo(Option<InstanceInfo>),
    Feed(FeedReply), Admin(AdminReply),
    Missing,                  // not held: the screen shows its loading branch and calls want()
    Unavailable(&'static str),// stateless paint: the screen's own unavailable branch, never a 500
}

pub trait Runtime: Send + Sync + 'static {
    fn revision(&self) -> Signal<u64>;
    fn read(&self, want: &Want) -> Reply;                                  // synchronous snapshot
    fn want(&self, want: Want, done: Callback<Result<Reply, String>>);      // start the read
    fn act(&self, action: Action, done: Callback<Result<Reply, String>>);   // replaces action/ActionResult
    fn now_millis(&self) -> f64;
    fn navigate(&self, _path: String) {}
}
```

`Missing` vs `Unavailable` is the distinction the current `Option` + `unavailable: bool` pair encodes by
hand in `TimelineReply`; making it explicit is what lets one screen body serve both paints without a
`cfg` (`docs/13-unification-plan.md`: "where the client can render more than the server, match the
client; Err is a state").

### Groups and the surface each needs

| # | group / new module | routes | `Want` | `Action` | slots the client keeps |
| --- | --- | --- | --- | --- | --- |
| 1 | `route::screens::timelines` (exists) | `Home, Local, Federated` | `Timeline(Home/Local/Federated)` | card verbs | `header`, `banner`, `card` |
| 2 | `route::screens::feed` | `Feed` | `Feed(People/Posts)` | `Follow`, `NotInterested` | `header`, `people`, `posts` |
| 3 | `route::screens::{notifications, messages, saved, directory, scheduled, search, people}` (5 exist) | `Notifications, Messages, Saved, Directory, Scheduled, Search, Mutes, Blocks` (+ followers/following on the profile) | `Page(PageKind::…)` (+ `FollowRequests` for the banner) | `AnswerFollowRequest`, `ReadConversation`, `MuteConversation`, `CancelScheduled`, `Mute{on:false}`, `Block{on:false}`, `Follow{on:true}` | `status_card`, `request_actions`, `banner`, `card` |
| 4 | `route::screens::lists` | `Lists` | `Lists`, `ListAccounts`, `Timeline(List(id))` | `CreateList`, `RenameList`, `DeleteList`, `SetListMember` | `row`, `members`, `timeline` |
| 5 | `route::screens::thread` | `Thread{host,id}` | `Status(id)`, `Context(id)` | `Vote` + card verbs | `card`, `reply_slot` |
| 6 | `route::screens::profile` | `Account(id)` | `Account(id)`, `Relationship(id)`, `Page(Followers/Following)`, `Timeline(Account(id))` | `Follow`, `Mute`, `Block` | `tabs`, `after`, `card` |
| 7 | `route::screens::compose` | `Compose`, `ComposeEdit(id)` | `Status(id)` only (for edit) | `Publish(NewStatus)`, `EditStatus{id,status}`, `Schedule{status,at}`, `UpdateMedia{id,description}` | `body`, `children` (editor, attachments, poll, schedule, drafts) |
| 8 | `route::screens::settings` | `Settings(section)` | `Session`, `InstanceInfo`, `Page(Filters)`, `Page(FeaturedTags)` | `UpdateProfile`, `CreateFilter`, `DeleteFilter`, `FeatureTag`, `UnfeatureTag` | `add_form` (filter/tag forms) |
| 9 | `route::screens::moderation` | `Moderation(section)`, `Mutes`, `Blocks` | `Admin(Staff)`, `Admin(Reports)`, `Page(Mutes/Blocks)` | `ResolveReport`, `ModerateAccount{id,action}`, `ApproveRegistration{id,accept}`, `BlockDomain{name,severity}`, `UnblockDomain` | — |
| 10 | `route::screens::admin` | `Admin(section)` | `Admin(Staff)`, `Admin(DomainAllows)`, `Admin(Measures)`, `Admin(Announcements)` | `AllowDomain`, `WithdrawDomainAllow` | — |
| 11 | session, not a screen | `OauthCallback`, signed-out `SignIn` | — | — (oauth is `web-client`/`web-server`, never `view`) | `SigningIn`/`SignInPage` already shared |

Two more `Route` changes are needed for this to type-check honestly:

1. **`Route::Search` needs its query** (`crates/web-shared/src/shared/router.rs:50` is a unit variant while
   the page needs `q`). Either `Search(String)` or a `RouteContext { path: String, query: Option<String> }`
   passed to `RouteView`.
2. **`Route::Account` needs a tab** (posts/followers/following) if the profile group is to own the
   followers list; today that is client-local `Tab` plus `Route::Account`.
3. `Route` also needs the `Directory(local_only)` scope if the server's `DirectoryPageProps::server` and the
   client's scope toggle are to agree on the URL; today the scope is a client signal only.

### Order of work that keeps the tree green

1. Add `want.rs` (`Want`, `Reply`, `FeedWant`, `FeedReply`, `AdminWant`, `AdminReply`) and give `Runtime`
   default `read`/`want` implementations that delegate to the existing methods — nothing breaks, no call
   site changes.
2. Teach `ClientRuntime` (`crates/web-client/src/state/runtime.rs:23`) to answer `read`/`want` from
   `bridge::{pages,timeline,actions,feed,posts,admin}`; delete the per-route bodies as each screen lands.
3. Add `StaticRuntime::with(replies: Vec<(Want, Reply)>)` and make `pages.rs` build the `Reply` it already
   has, then call `RouteView` instead of `page_of` (`crates/web-server/src/document.rs:365`) for
   `Notifications/Messages/Saved/Directory/Scheduled/Search/Lists`.
4. Convert `ProfilePage`/`ThreadPage` builders (`document.rs:267/321`) last: they are the only two that
   still assemble props inline, and `thread_page` already looks like the shared page's server call.
5. After each step, `cargo check -p nodus-view -p nodus-web-server` and
   `cargo check -p nodus-web-client --target wasm32-unknown-unknown` (`docs/13-unification-plan.md`).

## 4. Blockers (in the order they bite)

1. **Server has no non-timeline `Runtime`.** `StaticRuntime` (`crates/web-server/src/runtime.rs:12`) answers
   `timeline` only; every other URL goes `pages.rs:56-121` → `document::page_of` (`document.rs:365`), never
   `RouteView` (`crates/view/src/route/mod.rs:227`). The dispatcher cannot be the only path until
   `StaticRuntime` is constructed with the prefetched answers.
2. **Search query is out of band.** Client: `router::query_param("q")` (`crates/web-client/src/routes/search/mod.rs:31`).
   Server: URI query + `percent_decode` (`crates/web-server/src/pages.rs:198/244`). `Route::Search` carries
   nothing (`crates/web-shared/src/shared/router.rs:50`).
3. **Compose cannot be server-painted.** ProseMirror mount (`crate::ui::editor::RichEditor`), file input
   (`crates/web-client/src/routes/compose/attachments/mod.rs:38`), XHR upload with progress (:101),
   localStorage drafts (`compose/inline/drafts_panel/mod.rs:20`), emoji catalogue
   (`compose/inline/emoji/mod.rs:48`), client clock for the schedule grid (`compose/inline/schedule/mod.rs:19`).
   Keep `body`/`children`/`controls` client-supplied.
4. **Admin/moderation transports are browser-only.** `bridge::admin` (`crates/web-client/src/state/bridge/mod.rs:432`
   `keep_admin_token`, `:449` `admin_transport`, `:459` `admin_host`), grant via `oauth::begin_admin`
   (`crates/web-client/src/routes/settings/admin/mod.rs:6`). The server has `Client::is_staff` only, so
   `standing` must be a prop (`Some(StaffStanding)` / `None`) and `on_grant: None` on the server.
5. **`feature = "admin"` lives only in `crates/web-client/Cargo.toml`** (`admin = []`); `nodus-view` has no
   such feature while `pages::admin`/`pages::moderation` are unconditional. Moving `AdminScreen`/
   `ModerationScreen` into `view` means either adding the feature there or always compiling the screen with
   an inert `standing`.
6. **Load-only reads have no snapshot.** `ClientRuntime::follow_requests()` (`crates/web-client/src/state/runtime.rs:120`)
   and `lists()` (:127) return `None`, so the notification banner and the lists picker start empty; the
   server never prefetches follow requests at all (`pages.rs:57`). `Want`/`Reply::Missing` makes this
   explicit instead of an empty `None`.
7. **Screen-local state that must not become a `Want`:** muted conversation ids (`routes/conversations/mod.rs`),
   the selected list (`routes/lists/mod.rs:32`), busy/answering flags, `SyncUi { confirming, busy, result }`
   (`crates/web-client/src/routes/settings/sync/mod.rs:13`), drafts, the composer's signals.
8. **Browser-only affordances inside cards** must stay in the client's `CardRenderer`/slots: clipboard
   (`crates/web-client/src/routes/feed/share/mod.rs:53`), `window.confirm_with_message`
   (`routes/status/menu/mod.rs:29`), localStorage reveals (`routes/status/sensitive/mod.rs:11`), media
   lightbox, `remembered_host()` (`routes/status/mod.rs:37`, used by `thread_route`), impressions
   (`routes/feed/item/mod.rs:10`).
9. **`document::{profile_page, thread_page, shell}`** (`document.rs:267/321/395`) still build pages directly;
   `thread_page` passes `card: None`, `reply_slot: None`, so the shared screen must render without them
   (it already can — both are `Option`).
10. **Duplicated URLs during migration.** `/notifications`, `/messages`, `/saved`, `/directory`, `/scheduled`
    are now painted twice (client `screens/*` and `pages.rs`), which is fine only while both use the same
    component; any divergence is the bug `docs/13` says to avoid.
11. **Concurrent edits.** `crates/view/src/route/screens/{directory,messages,saved,scheduled}.rs` and
    `crates/web-client/src/state/runtime.rs` were written at 20:04–20:07 by another worker; coordinate
    before touching `route/mod.rs`, `runtime.rs`, or the screens directory.

### What is cheapest to convert next

`People` (one screen, four URLs, one `PageKind` per list), `Scheduled` and `Saved` (one page kind each,
no slots but `card`), then `Search` (only blocked on the query plumbing), then `Lists` (three panels,
three wants), then `Thread`/`Profile` (entity reads + `Context`), then settings, then compose, then
admin/moderation (blocked on 4/5).

TASK AS OF THIS BLOCK: matrix + props + grouping written to `docs/route-inventory.md`; remaining work is
the final reply (groups + blockers) to the operator. No source files were modified.
Open objectives: deliver the route inventory doc and the concise summary (m00001).
