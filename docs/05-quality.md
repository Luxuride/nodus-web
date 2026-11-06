# 05 — Quality: budget, tests, release

Status: draft · Last revised: 2026-11-06

"Blazingly fast" is only a claim until it is a number that fails the build.
This document holds the numbers.

## Performance budget

Measured on a mid-tier Android over throttled 4G, cold cache, brotli:

| Metric | Budget | Checked by |
| --- | --- | --- |
| WASM + JS transferred, initial route | ≤ 900 KB compressed | CI size gate |
| First contentful paint (app shell) | ≤ 1.0 s | Lighthouse budget, manual per milestone |
| Home timeline interactive, cached page 1 | ≤ 1.5 s | manual, milestone |
| Home timeline interactive, network | ≤ 2.5 s | manual, milestone |
| Dropped frames scrolling 300 items | < 5% on 60 Hz | manual trace, milestone |
| Memory after 30 min of scrolling | < 150 MB | manual, milestone |

Current baseline: the shell plus the SDK is **101 KB wasm / 38 KB gzipped** plus
25 KB JS, which leaves ~835 KB of budget for the real app — comfortable for
Leptos CSR, tight enough that careless dependencies are visible. Adding the SDK
cost ~75 KB raw (~13 KB gzipped); the next jump will be the transports.

Mechanisms:

- `trunk build --release` + `wasm-opt -Oz` (`--enable-bulk-memory`), `opt-level =
  "z"`, `lto = true`, `panic = "abort"` for the wasm profile.
- Route-level lazy components: thread, profile, settings and compose each load in
  their own chunk; the first route ships the shell, timeline, status card and feed.
- No icon font (inline SVG sprite), no analytics, no third-party scripts, no
  runtime CSS-in-Rust.
- Preload the wasm module from `index.html`; `Cache-Control: immutable` for
  hashed assets; a service worker precaches the shell for repeat visits.
- `cargo bloat` on demand when the gate trips; the gate is the authority.

The gate: CI builds, gzips, compares against the budget table, and **fails the
build** over budget. Raising a budget is a reviewed change to this file, not a
one-line CI tweak.

## Testing layers

| Layer | Tool | What it covers |
| --- | --- | --- |
| `mastodon-reccom-mastodon` (model) | `cargo test` | serde round-trips for statuses/notifications, `Page`/`PageParams` invariants |
| `mastodon-reccom-sdk` | `cargo test` (native) | the whole state machine: session/feed/timeline transitions, cache round-trip via `MemoryPersistence`, `FixedClock`, discovery-outage degradation |
| SDK ports (wasm impl) | `wasm-bindgen-test` + stub `fetch` | bearer header, stream URL construction, 401 path, IndexedDB adapter |
| `crates/web` logic | `wasm-bindgen-test` | sanitizer payload battery, outbox replay order, bridge wiring |
| API client | `wasm-bindgen-test` + stub | contract version header, `426` handling, error envelope mapping |
| UI flows | Playwright (dev-only Node) | sign-in (against a mock instance), timeline render, compose, feed follow, offline banner, keyboard shortcuts |
| Accessibility | Playwright + axe | key routes, each milestone |
| Visual | Playwright screenshots, manual review | theme and layout regressions |

**The mock instance** is a small axum fixture in this repository that implements
the endpoints the client uses (instance, apps, oauth token, timelines,
notifications, accounts, statuses, streaming stub). It makes CI deterministic,
lets us test hostile payloads (XSS, broken HTML, weird visibility), and is
reusable by backend integration tests. Playwright is the only Node dependency
and it never enters the shipped bundle.

## CI (this repository)

```text
fmt check → clippy --target wasm32-unknown-unknown -D warnings
         → cargo test (native + wasm-bindgen-test via wasm-pack/wasm-bindgen-test-runner)
         → trunk build --release → size gate → Playwright
         → cargo deny (licenses, advisories) → publish dist artifact
```

Nightly: build against the **latest contract tag** and against `contract@main`,
so pin drift surfaces on a schedule rather than at release. A `wasm-opt` version
bump is part of the lockfile review.

## Release

- Versioned deploys: `dist` published under `/v/<git-sha>` with `index.html`
  pointing at the current one; rollback is a pointer change, not a rebuild.
- Hashed assets immutable; `index.html` `no-cache`; strict CSP and the API's
  `WEB_ORIGINS` updated together in the deploy runbook.
- Contract compatibility: a release records the contract tag it was built
  against; the API's `min_contract` must be ≤ that tag
  ([08-sdk.md](../../backend/docs/08-sdk.md)).
- Sign-out and cache-clear paths are smoke-tested in every release, because they
  are the paths that only break in production.

## Risks and mitigations

| Risk | Mitigation |
| --- | --- |
| Full-client scope swallows the recommender | M2 is the cut line: timelines, thread, compose, notifications, feed. Parity waits ([01-scope.md](01-scope.md)). |
| WASM bundle exceeds budget | lazy routes from day one, gate in CI, `wasm-opt` in the pipeline |
| Instance differences (Akkoma, GoToSocial, older Mastodon) | capability probing from `/api/v2/instance`; degrade, never crash; fixture tests per capability set |
| Browser-held token stolen by XSS | strict CSP, no third-party scripts, sanitizer + payload tests, `SecretString`, sign-out revokes at the instance ([09-privacy.md](../../backend/docs/09-privacy.md)) |
| Leptos API churn | pin the version in `Cargo.toml`, upgrade deliberately per milestone, keep components thin over our state layer |
| Browser and worker rate limits collide | the worker self-limits to 50% of the reported budget and backs off when headers show the browser consuming it ([04-ingestion.md](../../backend/docs/04-ingestion.md)) |
| Nobody switches clients (so nobody sees the Feed) | measured, not assumed: M3 exit criterion is a 5-person pilot using it as their primary client for two weeks |
