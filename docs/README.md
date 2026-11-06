# Design documentation — client

These documents describe the **client**: a complete Mastodon web client in Rust
(Leptos → WebAssembly) whose reason for existing is the **Feed tab**.

Status: **draft** — written before implementation. Each document carries its own
status line.

The backend half of the design lives in [`backend/docs/`](../../backend/docs/README.md).
The two meet at the SDK (`sdk/`), which also **owns the client's state**: this
client renders SDK state and dispatches commands. It has its own README
([`sdk/README.md`](../../sdk/README.md)) and its rules are set out in
[`backend/docs/08-sdk.md`](../../backend/docs/08-sdk.md).

```text
MastodonReccom/          ← superproject (submodules, .devcontainer)
├── sdk/          mastodon-reccom-shared, -mastodon, -sdk · README
├── backend/      core, store, ingest, api · docs/
└── web/          this repository: crates/web · docs/ ← you are here
```

## Reading order

| Doc | Contents |
| --- | --- |
| [01-scope.md](01-scope.md) | What the client is, why Leptos, parity matrix, non-goals |
| [02-architecture.md](02-architecture.md) | Crate layout, routes, state ownership, streaming, deployment |
| [03-data-layer.md](03-data-layer.md) | Contract client, transports, token/PKCE, IndexedDB cache, optimistic updates |
| [04-ui.md](04-ui.md) | Rendering rules, sanitization, virtualization, theming, a11y, i18n |
| [05-quality.md](05-quality.md) | Performance budget, testing, CI, release, risks |

## The two claims this repository must satisfy

1. It is a client people will actually use daily — not a demo shell.
2. The Feed tab is faster and more useful than scrolling a timeline.

If (1) holds but (2) does not, we built a Mastodon clone with a mediocre tab. If
(2) holds but (1) does not, nobody ever sees the recommender. The scope order in
[01-scope.md](01-scope.md) is chosen so neither bet is taken blind.

## Conventions

- Markdown, one H1 per file, numbered files, `Status:` line at the top.
- Backend behaviour is **linked, never restated**: ranking and similarity live in
  `backend/docs/`, the wire format and client state in `sdk`.
- Any statement about "the instance" means a third-party Mastodon-compatible
  server we do not control; treat its responses as untrusted input.

## Open questions

1. **Instance software matrix.** Which non-Mastodon servers must work in M2
   (Akkoma, GoToSocial, Pleroma)? Assumed: Mastodon ≥ 4.2 is the only *supported*
   target at M2, with graceful degradation elsewhere.
2. **Embedded media policy.** Proxy nothing (status quo, recommended) vs a
   privacy-preserving image proxy. Assumed: no proxy; direct media with
   `Referrer-Policy: no-referrer`.
3. **Hosting for `dist`.** Any static host/CDN works; the constraint is a
   wildcard-free CSP and immutable hashed assets. Assumed: static host, versioned
   deploys, SPA fallback to `index.html`.
4. **PWA vs installable app.** Assumed PWA only (no app-store packaging) until
   M5.
