# One renderer: finishing the SSR/CSR unification

Status: **in progress**. `nodus_view` already owns the frame, the rail, the icons, the
timeline tabs, the composer card, the page header, the screen column, the notification row
and the media rules. `web/crates/server/src/render.rs` still builds markup as strings for the
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

- `cargo clippy -p nodus-view -p nodus-server --all-targets -- -D warnings`
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

Still strings in `crates/server/src/render.rs`: `profile_page`, `thread_page`,
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
