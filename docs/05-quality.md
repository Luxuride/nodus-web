# 05 — Quality: budget, tests, release

Status: draft · Last revised: 2026-11-06

"Blazingly fast" is only a claim until it is a number that fails the build.
This document holds the numbers.

## Performance budget

Measured on a mid-tier Android over throttled 4G, cold cache, brotli:

| Metric | Budget | Checked by |
| --- | --- | --- |
| WASM + JS transferred, initial route | ≤ 900 KB compressed | `gzip -9` on `dist/`, recorded per release |
| First contentful paint (app shell) | ≤ 1.0 s | Lighthouse budget, manual per milestone |
| Home timeline interactive, cached page 1 | ≤ 1.5 s | manual, milestone |
| Home timeline interactive, network | ≤ 2.5 s | manual, milestone |
| Dropped frames scrolling 300 items | < 5% on 60 Hz | manual trace, milestone |
| Memory after 30 min of scrolling | < 150 MB | manual, milestone |

Current baseline (the whole client: timelines, threads, profiles, composer with
upload, scheduling and polls, notifications, messages, saved, lists, directory, search and
seven settings sections): **2.00 MB wasm / 728 KB gzipped** for the default build, plus
50 KB JS (10 KB gzipped) and 219 KB CSS (36 KB gzipped) — **774 KB transferred**, inside
the 900 KB budget with ~126 KB to spare.

The moderation screens are a Cargo feature, `admin`, off by default: they cost 261 KB of
wasm (81 KB gzipped), which is the difference between the two measured builds —
`trunk build --release` gives 2 003 335 B (727 775 B gzipped) and
`trunk build --release --features admin` gives 2 265 107 B (808 349 B gzipped). A
moderator build is the second command; `https://nodus.luxuride.eu` is deployed from it,
because its operator moderates. Everything else — including the muted and blocked lists,
which belong to the account rather than to moderation — is in both builds. The budget was measured when the client had five screens;
it held, and the number is written here so the next feature has to argue with it. What moved it: the instance surface (thirty-odd
endpoints and their models), `serde_json`, the `web_sys` bindings, and the admin screens.
The CSS is daisyUI's used-component set, which is where the 217 KB goes and why themes
are capped at two. Shrinking it further means feature-gating the admin and settings
screens, not micro-optimising the renderer.

Traps that move the number in ways nothing explains, recorded because each cost an hour:

- **Never call a function containing a spawned `loop` inside a `#[component]` body before
  its `view!`.** Doing that made the whole UI unreachable for LTO: the bundle *shrank* by
  224 KB and every screen's markup vanished from the wasm while `clippy` stayed clean. The
  tick is therefore started from `main`, after `mount_to_body`, and the size baseline is
  re-checked whenever startup code moves.
- **Check for a stale `dist` before believing a measurement.** Trunk can report success
  without rewriting artefacts after a source edit; `rm -rf dist && trunk build --release`
  before recording a number, and grep the wasm for one string you know the new code added.
- **String presence is a weak but useful signal**: `grep -c "Background sync is on"
  dist/*_bg.wasm` returning 0 once proved the UI had been dropped.

Mechanisms:

- `trunk build --release` + `wasm-opt -Oz`, `opt-level = "z"`, `lto = true`,
  `codegen-units = 1`, `panic = "abort"` in `[profile.release]`.
- **`strip` stays off**: stripping removes the `target_features` custom section,
  wasm-bindgen can no longer tell which Wasm features are enabled, and `wasm-opt`
  then fails validation on the `memory.copy` it still emits (`wasm-validator
  error … memory.copy operations require bulk memory operations`).
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

### Measured alternatives (2026-11-07)

Every number below is the same pipeline (`cargo build --release`, then
`wasm-bindgen --target web`, then `wasm-opt -Oz`), so the deltas are real:

| Variant | wasm raw | wasm gzipped | gzipped delta |
| --- | --- | --- | --- |
| stable, current profile | 485 367 | 205 649 | — |
| nightly `-Zbuild-std=std,panic_abort` + `-Cpanic=immediate-abort` | 440 541 | 188 242 | **−8.5%** |

Notes that cost time to learn:

- `-Zbuild-std` is a **cargo** flag, not a `RUSTFLAGS` one: putting it in
  `RUSTFLAGS` fails with `error: unknown unstable option: build-std`.
- `-Zbuild-std-features=panic_immediate_abort` no longer works; current nightlies
  say `panic_immediate_abort is now a real panic strategy! Enable it with
  panic = "immediate-abort"` — i.e. `-Cpanic=immediate-abort` plus `-Zbuild-std`.
- **Not adopted.** 17 KB gzipped does not buy a permanently pinned nightly, a
  `rust-src` component and a second toolchain in every CI job, on a bundle that
  already sits at 27% of its budget. Revisit if the gate gets tight.
- `serde_json` cannot be removed by removing our own use of it: it arrives through
  `gloo-net` (and `leptos_server` via `codee`/`server_fn`). Dropping it means
  dropping `gloo-net` for hand-rolled `web_sys::fetch` transports, which is the
  single largest remaining lever and a refactor, not a flag.

### Why Trunk and not `cargo-leptos`

For a CSR-only app the two run the same pipeline: build wasm, run `wasm-bindgen`,
run `wasm-opt`, hash assets. `cargo-leptos` adds what we deliberately do not use —
the SSR/hydration dual build, server functions, and `cargo leptos watch`'s reload
— while replacing a working `Trunk.toml` with a `[package.metadata.leptos]`
section and a `site-root` layout. Neither tool changes the numbers above, because
both call the same `rustc`, the same profile and the same `wasm-opt -Oz`. If SSR
lands for public pages (see `02-architecture.md`, D8), `cargo-leptos` becomes the
right tool, because at that point there are two builds to keep in step.

## Testing layers

| Layer | Tool | What it covers |
| --- | --- | --- |
| `nodus-instance` (model) | `cargo test` | serde round-trips for statuses/notifications, `Page`/`PageParams` invariants |
| `nodus-sdk` | `cargo test` (native) | the whole state machine: session/feed/timeline transitions, cache round-trip via `MemoryPersistence`, `FixedClock`, discovery-outage degradation |
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
reusable by recommender integration tests. Playwright is the only Node dependency
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
  ([03-contract.md](../../sdk/docs/03-contract.md)).
- Sign-out and cache-clear paths are smoke-tested in every release, because they
  are the paths that only break in production.
- Naming and trademarks: the about screen shows the "not affiliated with Mastodon
  GmbH" disclaimer, `dist` bundles no Mastodon logo or instance artwork, and no
  new route, endpoint or asset name reintroduces the mark
  ([11-trademark.md](../../recommender/docs/11-trademark.md)).

## Risks and mitigations

| Risk | Mitigation |
| --- | --- |
| Full-client scope swallows the recommender | M2 is the cut line: timelines, thread, compose, notifications, feed. Parity waits ([01-scope.md](01-scope.md)). |
| WASM bundle exceeds budget | lazy routes from day one, gate in CI, `wasm-opt` in the pipeline |
| Instance differences (Akkoma, GoToSocial, older Mastodon) | capability probing from `/api/v2/instance`; degrade, never crash; fixture tests per capability set |
| Browser-held token stolen by XSS | strict CSP, no third-party scripts, sanitizer + payload tests, `SecretString`, sign-out revokes at the instance ([08-privacy.md](../../recommender/docs/08-privacy.md)) |
| Leptos API churn | pin the version in `Cargo.toml`, upgrade deliberately per milestone, keep components thin over our state layer |
| Browser and worker rate limits collide | the worker self-limits to 50% of the reported budget and backs off when headers show the browser consuming it ([04-ingestion.md](../../recommender/docs/04-ingestion.md)) |
| Nobody switches clients (so nobody sees the Feed) | measured, not assumed: M3 exit criterion is a 5-person pilot using it as their primary client for two weeks |

## Composer assets (2026-11-08)

The body editor and the emoji picker are two vendored bundles, built with `npm run build` in
`web/js` and committed under `web/crates/web/public/vendor/`. Nothing in the Rust build or the
deployment needs npm; rebuilding them is a deliberate act:

| file | raw | gzipped | loaded |
|---|---|---|---|
| `vendor/editor.js` (ProseMirror + schema) | 206 KB | 64 KB | when a composer mounts |
| `vendor/picker.js` (`emoji-picker-element`) | 38 KB | 14 KB | first time the picker opens |
| `vendor/emoji-data.json` | 430 KB | ~120 KB | first time the picker opens |

Measured budget consequence: the first paint of `/` is unchanged — both bundles are fetched
only by a signed-in composer — and the composer adds 64 KB of JavaScript, not wasm.

Why ProseMirror at all: a `:shortcode:` has to be drawn as the picture it will become, with
the caret still in the text. A `textarea` cannot, and an overlay mirror drifts from the caret.
The schema deliberately has no bold, links or headings: the API accepts plain text, and an
editor that can express more than it can send lies about the post.
