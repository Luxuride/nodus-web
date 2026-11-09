# Generated assets

Three browser-side artifacts exist that are not in this repository, because they are
compiled from sources that are:

| Artifact | Built from | By |
| --- | --- | --- |
| `crates/web/style/tailwind.css` | `style/app.css` + Tailwind/daisyUI from npm | `js/css.mjs` |
| `crates/web/public/vendor/editor.js` | `js/editor.mjs`, `js/schema.mjs`, ProseMirror | `js/build.mjs` (esbuild) |
| `crates/web/public/vendor/picker.js` | `js/picker.mjs`, `emoji-picker-element` | `js/build.mjs` (esbuild) |
| `crates/web/public/vendor/emoji-data.json` | `emoji-picker-element-data` (en) | copied by `js/build.mjs` |

`crates/web-client/build.rs` runs those scripts when an artifact is missing or older than
its source, so `cargo build`/`trunk build` produce them and a stale bundle cannot ship.
`web/.gitignore` keeps the outputs out of the tree.

## Why npm rather than a vendored copy

Tailwind v4 resolves `@import "tailwindcss"` and `@plugin "daisyui"` through `node_modules`
above the CSS file, so the CSS entry point has to live in a directory with `node_modules`.
`package.json` and `package-lock.json` pin the versions; nothing else is taken from the
network.

## Measured sizes (2026-11-09)

| File | Raw | Gzipped |
| --- | --- | --- |
| `editor.js` | 206 KB | 64 KB |
| `picker.js` | 38 KB | 14 KB |
| `emoji-data.json` | 430 KB | — |
| `tailwind.css` | 148 KB | 24 KB |

## Updating

```sh
cd web
npm install                      # once, or after a lockfile change
npm run build                    # both bundles and the stylesheet
```

Then rebuild: `trunk build --release --features admin`. A build without Node fails with a
message naming this file, which is deliberate — a composer that renders without its editor
is worse than a build that stops.
