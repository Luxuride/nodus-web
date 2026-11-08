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
