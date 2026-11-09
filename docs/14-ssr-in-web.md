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
ssr     = ["leptos/ssr", "dep:tokio", "dep:hyper", "dep:nodus-instance/native"]
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
