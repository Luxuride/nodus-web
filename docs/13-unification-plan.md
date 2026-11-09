# One renderer: finishing the SSR/CSR unification

Status: **in progress**. `nodus_view` already owns the frame, the rail, the icons, the
timeline tabs, the composer card, the page header, the screen column, the notification row
and the media rules. `web/crates/web/src/server/render.rs` still builds markup as strings for the
components below.

The goal is one definition per component, rendered by `tachys::view::RenderHtml::to_html()`
on the server and mounted by the client. A component may not read a transport, storage, the
URL or the clock: those arrive as props, so the server can pass `None` where an interaction
does not exist.

## The rule for interactions

Every interactive affordance is an optional callback:

```rust
#[component]
pub fn StatusFooter(
    status: Status,
    age: String,
    #[prop(optional, into)] on_favourite: Option<Callback<String>>,
    #[prop(optional, into)] on_boost: Option<Callback<String>>,
    #[prop(optional, into)] on_bookmark: Option<Callback<String>>,
    #[prop(optional, into)] on_open: Option<Callback<String>>,
) -> impl IntoView
```

A button renders whenever its callback exists and renders as a link or plain text when it does
not, so the **markup is identical** and only the affordance differs. `None` on the server is
not a degraded paint; it is the same card a signed-out reader sees.

## Order of work (one commit and one deploy per step)

1. `StatusBody` — content warning, body HTML, emoji, media grid, poll.
   - Source: `web/crates/web/src/routes/status/body.rs` (328 lines) and
     `server/src/render.rs::status_card`'s inner part.
   - Props: `status: Status`, `blurred: bool`, `on_reveal: Option<Callback<()>>`,
     `on_open_media: Option<Callback<(String, String)>>`, `on_vote: Option<Callback<Vec<usize>>>`.
   - The poll needs `on_vote`; on the server the poll renders read-only with the same bars.
2. `StatusFooter` — counts, age, edit marker, the three verbs, the owner menu.
   - On the server only counts and age render; the verbs take `None`.
3. `StatusHeader` — avatar, display name, handle, timestamp link. No callbacks: it is a link.
4. `StatusCard` — boost unwrapping, then header + body + footer. `web/crates/web/src/routes/status/mod.rs`
   shrinks to: read the signal, call `<nodus_view::StatusCard …/>` with callbacks.
5. The rows: `AccountRow`, `ConversationRow`, `ListRow`, `ScheduledRow` move out of
   `server/src/render.rs` and are used by `routes/people.rs`, `routes/conversations.rs`,
   `routes/scheduled.rs`, `routes/lists.rs`.
6. `ProfilePage` and `ThreadPage` — the last two, because they are mostly composition of 1–4.
7. Delete what is left of `server/src/render.rs` string helpers and add a test that asserts a
   page's HTML is produced by `nodus_view` components (e.g. a marker `data-view="status-card"`
   that only the component emits).

## Verification per step

- `cargo clippy -p nodus-view --all-targets -- -D warnings`
- `cargo clippy -p nodus-web --no-default-features --features ssr --all-targets -- -D warnings`
- `cargo clippy -p nodus-web --target wasm32-unknown-unknown --all-targets --features admin -- -D warnings`
- `cargo test -p nodus-view`
- `trunk build --release --features admin`, then `curl` the affected route and diff the markup
  the server emits against the classes the client renders.
- `npx playwright test` in `web/e2e` (see the open failures in `docs/05-quality.md`).

## What "done" looks like

`server/src/render.rs` contains only: asset discovery, the document skeleton, the session
reads and the island payload. No HTML strings. A grep for `format!("<` in the server crate
returns nothing.

## The rule for missing data (settled 2026-11-09)

Where the client can render more than the server, **match the client** — a different card for
the same status is the bug this document exists to remove. Where the data is not available at
all, the server's stateless read returns `Err`, and that `Err` is handled as a *state*: the
screen paints the same component with the fields it has and the branch that needs the missing
read (a poll's tally, another account's relationship) renders its unavailable form inside the
markup the component already owns. It never falls back to a different hand-built string.

Push notifications are explicitly **last in scope**; nothing in this refactor should assume
they exist.

## State (2026-11-09)

Shared and wired: `Frame`, `Rail`, icon set, `Screen`, `PageHeader`, `ComposerCard`,
`TimelineTabs`, `NotificationCard`, media rules, `MediaGrid`, `StatusHeader`, `StatusFooter`.

Shared but **not yet wired** (declared in `nodus-view`, no call sites): `lists::AccountRow`,
`lists::ConversationRow`, `lists::ListRow`, `lists::ScheduledRow`, `chrome::MoreButton`,
`chrome::Skeleton`, `chrome::TimelineHeader`.

Written but not compiled (module declarations deliberately removed): `status_body.rs`,
`status_card.rs` — `StatusBody` moves `Status` into several `view!` closures, so each captured
field needs a signal or `StoredValue` before the `FnOnce` closures compile.

Rule settled by three reviews: **the row owns its boxes, the screen owns its verbs.** A verb
that is the screen's (list undo, list rename/delete, the post menu, share) arrives as a
`ChildrenFn` slot or is simply not the row's prop; a verb the row can justify (open, select,
cancel) arrives as an `Option<Callback<..>>` and renders disabled-but-identical when absent.

Still strings in `crates/web/src/server/render.rs`: `profile_page`, `thread_page`,
`timeline_header`/`page_header` adapters, `tabs_for`, `skeleton`, `more_button`.

## Decision (2026-11-09): everything in nodus

The operator's words: *"Everything in nodus, also move web routes/ to view."* So the target layout
is not "shared atoms only" — it is:

- `web/crates/view/src/components/` — pieces with no screen of their own: rows, cards, chrome,
  media, icons.
- `web/crates/view/src/pages/` — one module per screen, rendering the whole page body.
- `web/crates/web/src/routes/` — thin shells: read the SDK, hold signals, pass callbacks to a
  `nodus_view::pages::…` component. No markup beyond the wiring.

The line that decides what may stay in `web/` is not "is it small" but "does it own state". A
ProseMirror mount, the media modal's open signal, and the page store's cursors are state, so they
stay in the client and reach a page through props (`Signal`, `Callback`, `ChildrenFn` slots). A
page never fetches, never reads a clock, never navigates by itself.

### Consequence: SSR renders pages, not strings

Once a screen is a `nodus_view::pages::…` component, the server's job stops being "build HTML
that resembles the client" and becomes "read the data, render the page". It reads through the
SDK's fallible `Reads` (`Headless`), so every read is a `Result`:

- `Ok(data)` → the page with data, `on_*: None`, `interactive: false`.
- `Err(e)` → the *same* page component in its unavailable branch — the state the client shows
  when the same read fails — never a different hand-built string, and never a 500 page for a
  page whose data merely could not be fetched.

`render.rs` then keeps only what is genuinely server-side: asset discovery, the document shell,
the island JSON, and the two or three page-level guards (signed out, staff only).

### The `pages/` tree mirrors the URL tree

A page module is named after the path that renders it, so a stale filename cannot survive a
rename and a reader can find the code for `/local` without a search:

```
pages/index.rs            /
pages/feed.rs             /feed
pages/local.rs            /local
pages/federated.rs        /federated
pages/notifications/      /notifications        (row, request, banner)
pages/messages.rs         /messages
pages/saved/              /saved, /saved/{favourites,bookmarks}
pages/lists/              /lists, /lists/{id}    (members, timeline)
pages/scheduled.rs        /scheduled
pages/directory.rs        /directory
pages/search/             /search               (results)
pages/accounts/           /accounts/{id}
pages/status/             /status/{id}
pages/compose/            /compose, /compose/{id}
pages/settings/           /settings/{section}
pages/moderation/         /moderation/{section}
pages/admin/              /admin/{section}
```

`pages/status/public.rs` is the shared-post document at `/s/…` — a server-rendered page, not the
client's status route, and it keeps its own name because its URL says so.

## State (2026-11-09, second pass)

Shared and wired: every settings card (`components::panel::Panel`), the administration and
moderation areas (`pages::admin`, `pages::moderation`), the composer (`pages::compose`), the reply
box (`pages::status::reply_card`), the status menu, the feed page and its cards
(`pages::feed::{FeedPage, posts::PostCard, accounts::{Identity, ReasonBlock, AccountCard}}`), the
timeline strip (`components::timeline_tabs`), and `pages::timeline::StatusList` — the list, its
keys, its empty note and its "Load more", with the post itself supplied as a
`components::card_renderer::CardRenderer`.

The three traps that cost the most compile cycles, now encoded in the code's comments:
`Option<Callback<..>>` props need `#[prop(optional_no_strip)]` (plain `optional` strips the
`Option` and the setter then demands a bare callback, so `None` becomes an enabled button); a
`String` read inside a `Show`/`fallback` closure must sit in a `StoredValue`, and an `Arc` called
inside `<For>` must be `StoredValue::new(arc)` read with `with_value`, or the closure is `FnOnce`
(`error[E0525]`); and a component that forwards a caller's optional verb passes it straight through
— unwrapping at the call site with a no-op `Callback` is what breaks the chain.

## State (2026-11-09, final pass)

Every CSR screen is now a `nodus_view` page or component. `crates/web-client/src` holds no raw
HTML element tags: its remaining `view!` calls compose shared components, hold signals, and pass
callbacks. The last markup moved out was:

- the schedule panel and its month grid (`pages::compose::inline::schedule`, with the client still
  supplying the clock, the quick answers and the grid's cells);
- the emoji hint list (`pages::compose::inline::hints`), the poll editor, the language picker,
  the media lightbox, and the composer's popover/anchor/field boxes;
- the settings cards (`pages::settings::{SessionCard, AccountsCard, FeedsCard, FiltersCard,
  FeaturedTagsCard, ClockCard, SensitiveCard, InstanceCard}`) and the settings shell
  (`SettingsPage`, `SettingsSection`, `SettingsTabs`);
- sign-in (`pages::sign_in::SignInPage`), the landing body, the timeline header placeholder, and
  the shell's error body.

The server no longer builds page markup by hand either. `crates/web-server/src/document.rs` keeps
only the document envelope (doctype, assets, island, `<div id="app">`), the rail's extra staff
doors (through `RailEntry`), and status bodies (sanitized instance HTML). The signed-out shell
mounts the real `SignInPage` with every verb absent; an unknown route mounts `ShellPage`; a failed
timeline read mounts `TimelineError`; a painted timeline wraps its shared cards in `StatusColumn`.
`content.rs` still writes the `<img>` for a custom emoji because that is a rewrite of instance
content, not a view.

Verification: `cargo check -p nodus-view -p nodus-web-server`, `cargo check -p nodus-web-client
--target wasm32-unknown-unknown`, `cargo test -p nodus-view -p nodus-web-server` (25 tests), and
clippy on the same targets are clean.

## Route renderer (2026-11-09, in progress)

Parity is not the same as sharing components. Two screens can be made of the same widgets and
still disagree about which widgets a route has, which state it starts in, and whether a verb is
present. The route layer is the next shared boundary:

- `nodus_view::route::Runtime` is the platform contract. Reads are synchronous snapshots
  (`Runtime::timeline`), loads report completion through a `Callback`, and a stateless paint
  answers `TimelineReply::unavailable()` for a page it did not prefetch. The browser answers from
  the SDK store; the server answers from the page it already fetched.
- Callbacks are **always present**. `nodus_view::route::noop()` supplies the server's verbs, so a
  button keeps its markup and loses only its effect. A missing callback is no longer used to mean
  "the server painted this".
- `nodus_view::route::RouteView` is the router both paints call. It currently dispatches the three
  primary timelines (`Home`, `Local`, `Federated`) through a shared `TimelineScreen`; the client
  supplies its rich composer and card slots, the server supplies the same page with a
  `StaticRuntime` and no-op navigation. `TimelinePage` is now reached through this layer on both
  sides instead of the client wrapper and the server's `StatusColumn` taking different paths.

Dispatched today: the three primary timelines, notifications (including follow requests),
messages, saved shelves, directory, scheduled, search, thread, account (posts, followers,
following, follow/mute/block), mutes, blocks, and lists (picker, members, add-by-search, list
timeline). The client wrappers for those routes are gone; the server paints every one of them
through `RouteView` with a prefetched `StaticRuntime`, and unknown pages fall back to the same
"Not available" screen. The server's `profile_page` and `thread_page` builders are gone.

Still client-only by nature, not by omission: the algorithmic feed (`/feed` reads the local
recommender and records impressions; the server has no feed), compose (ProseMirror mount, uploads,
drafts, local clock), settings (per-card local state and localStorage), moderation/admin (the
server has no admin token), and the anonymous `/s/{host}/{id}` public thread (its host travels in
the URL and is read anonymously). These routes keep their client handlers until the platform
contract grows a snapshot for their reads; they are the remaining difference between an SSR paint
and a mounted paint. The shared `StatusCard` now receives no-op callbacks on the server instead of
absent ones, so its verbs keep their markup on both paints.
