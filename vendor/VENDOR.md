# Generated assets

Nothing here is a copy of a package any more: the stylesheet and the composer bundles are
built from `web/package.json` (npm), and only their **outputs** are committed so that
`trunk build` works on a machine without Node or a network.

## `crates/web/public/vendor/`

| file | made by | why committed |
| --- | --- | --- |
| `editor.js` (206 KB / 64 KB gz) | `npm run build` → `js/build.mjs` (esbuild) | the composer's ProseMirror body; a Rust build cannot produce it |
| `picker.js` (38 KB / 14 KB gz) | same | `emoji-picker-element`, mounted on demand |
| `emoji-data.json` (430 KB) | `js/fetch-emoji.mjs`, from `emoji-picker-element-data` | self-hosted so the picker never contacts a CDN |
| `../../style/tailwind.css` (283 KB / 41 KB gz) | `npm run css` → `js/css.mjs` (Tailwind CLI from `node_modules`) | every page's stylesheet; git-tracked, not ignored |

## Updating

```sh
cd web
npm install          # pinned by package-lock.json
npm run build        # bundles + stylesheet
```

Commit the regenerated files. `Tailwind.toml`'s hook that used to run a standalone
`tailwindcss` binary is gone: the CLI now resolves `tailwindcss` and `daisyui` from
`web/node_modules`, so the two versions move together in `package.json`.
