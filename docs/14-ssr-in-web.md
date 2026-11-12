# Serving the app from the app crate

Operator decision (2026-11-09): *"Use web also for SSR, keep view crate as it is nice separation."*
So the three crates become two roles:

| crate           | role                                                                                                                                                  |
| --------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| `crates/view`   | `nodus-view`: every component and page; no I/O, no clock, no router; compiles for both targets                                                        |
| `crates/web`    | `nodus-web`: the app — client entry behind `hydrate`, server entry behind `ssr`                                                                       |
| `crates/server` | `nodus-server`: retired once the `ssr` entry point is proven; until then it is the working renderer and the reference for what the server needs        |

## Why this is now cheap

The expensive part of SSR is shared markup, and it is already done: pages and components live in
`nodus-view` and are rendered on both sides today (`tachys::view::RenderHtml::to_html` inside
`Owner::new().with(…)`). What is left is plumbing — one crate compiling twice instead of two
crates with duplicated glue.

## Shape

```toml
# crates/web/Cargo.toml
[features]
default = ["hydrate"]
hydrate = ["leptos/hydrate", "dep:wasm-bindgen", "dep:web-sys"]
ssr     = ["leptos/ssr", "dep:tokio", "dep:hyper", "dep:nodus-client-core/native"]
admin   = []
```

- `src/main.rs` keeps only what both targets share (module declarations plus the `App` composition
  is already in `nodus-view`); the client entry becomes `src/bin/client.rs` last in the sequence,
  because `trunk` builds `main.rs` and moving it is a one-line config change.
- `src/server/` (new, `#[cfg(feature = "ssr")]`): the session store, the Mastodon reader and the
  route table — moved from `crates/server/src/{session,mastodon,html}.rs`, unchanged in behaviour.
- `src/bin/server.rs` (`#[cfg(feature = "ssr")]`): document shell, asset discovery, island JSON,
  then `nodus_view::pages::…` per route.

## The one real cost: 47 modules touch the browser

`grep -rl "web_sys\|wasm_bindgen\|gloo" crates/web/src` matches 47 files. Every one must be
gated `#[cfg(feature = "hydrate")]` (or compile clean on `ssr`, which most cannot): `main.rs`,
`router.rs`, `media.rs`, `platform/`, `public.rs`, `state/bridge/**`, `ui/editor.rs`,
`routes/**` where a handler touches the DOM, `compose/**`, `settings/**`.

Gating is mechanical and it is the whole of the work. Rules that keep it mechanical:

1. Gate at the module declaration, never inside a function — one `#[cfg(feature = "hydrate")]` per
   file, in the parent `mod` list.
2. Anything a page needs from the browser arrives as a prop; if a page needs a browser API
   directly, the page is wrong (that is the `nodus-view` contract, already enforced).
3. `state/bridge` is client-only in full: the server reads through the SDK's `Reads` and never
   needs the page store.

## Sequence

1. Land the five in-flight page workers; green the tree.
2. Add the features and the `ssr`/`hydrate` split, gate the 47 modules, keep `trunk build` working.
3. Move `session.rs`/`mastodon.rs`/`html.rs` into `crates/web/src/server/`; build `nodus-web-server`
   from the same crate; run it against the live instance and diff its HTML against `nodus-server`'s.
4. Swap the systemd unit, keep `nodus-server` binary on disk one release, delete `crates/server`.
5. Optional, only once (4) is stable: real hydration (`leptos/hydrate`) instead of the current
   mount-and-replace — that removes the second paint entirely, but it needs `leptos_router`, which
   our `router.rs` replaces today.

## Done (2026-11-09)

`crates/server` no longer exists; the table and the sequence above are the plan, this is what it
became. One crate, three folders, two artifacts:

- `crates/web/Cargo.toml`: features `default = ["hydrate"]`, `hydrate` (browser deps, `leptos/csr`,
  `nodus-client-core/wasm`), `ssr` (axum, tokio, lol_html, tracing, `nodus-client-core/native`), and
  `admin`, which composes with either. Two bins: `nodus-web` (`src/main.rs`, `required-features =
  ["hydrate"]`) and `nodus-web-server` (`src/bin/nodus-web-server.rs`, `required-features =
  ["ssr"]`). `"crates/server"` left the workspace `members`.
- `src/lib.rs` is the seam: `#[cfg(feature = "hydrate")] pub mod browser;`, `pub mod shared;`,
  `#[cfg(feature = "ssr")] pub mod server;`, plus re-exports so the pre-split names
  (`crate::state`, `crate::router`, …) still resolve inside the client.
- `src/browser/` (hydrate only): `app.rs`, `boot.rs`, `media.rs`, `platform/`, `public.rs`,
  `router/` (the route signal, the delegated click listener, `navigate`, `link`, `install`,
  `query_param`), `routes/`, `scroll.rs`, `shell.rs`, `state/`, `theme.rs`, `ui/`. The old
  `main.rs` body is `boot::start`; `main.rs` is now `fn main() { nodus_web::start(); }`.
- `src/shared/router.rs` (both builds): `Route`, `path()`, `from_path()`, the section defaults,
  `BASE`, and `CALLBACK_PATH`, which `state::oauth` re-exports so the registered redirect URI and
  the address-bar route cannot drift apart.
- `src/server/` (ssr only): `session.rs`, `mastodon.rs`, `html.rs` (with its six tests) and
  `render.rs`, moved byte-identically apart from `crate::html` → `crate::server::html`.
- `src/bin/nodus-web-server.rs`: the old `crates/server/src/main.rs`, with its `mod` tree replaced
  by `nodus_web::server::…` imports and the tracing filter renamed to `info,nodus_web=debug`.

```sh
trunk build --release                    # the client: hydrate is the default
cargo build -p nodus-web --no-default-features --features ssr --bin nodus-web-server --release
cargo clippy -p nodus-web --no-default-features --features ssr --all-targets -- -D warnings
cargo clippy -p nodus-web --target wasm32-unknown-unknown --all-targets --features admin -- -D warnings
```

Step 5 is still open: the client still mounts and replaces, it does not hydrate.

## Landed (2026-11-09)

The sequence below was executed, with one change of shape: rather than one package holding
two binaries, each artifact has its own package.

| crate | kind | features | artifact |
| --- | --- | --- | --- |
| `crates/view` | lib | none | components both paints render |
| `crates/web` | lib | `hydrate` (default), `ssr`, `admin` | the app: `browser/`, `server/`, `shared/` |
| `crates/web-client` | bin | `hydrate`, `admin` | `nodus-web` wasm, built by `trunk` |
| `crates/web-server` | bin | `ssr` (default), `admin` | `nodus-web-server` |

The reason for two packages instead of two `[[bin]]`s in one: Cargo unifies features per
build, so `cargo build --workspace` on the single package enabled `hydrate` and `ssr`
together and the app crate then compiled with `leptos/csr` *and* `leptos/ssr` — which does
not build. One paint per package makes the workspace build safe, and `--features admin`
means the same thing to every crate in the chain.

`crates/server` is gone; `session`, `mastodon`, `html` and `render` live under
`crates/web/src/server/`, and the SSR host is 554 lines of axum in
`crates/web-server/src/main.rs`.
