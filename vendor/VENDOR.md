# Vendored dependencies

## `daisyui/` — daisyUI 5.1.10

Tailwind CSS component classes (`btn`, `card`, `modal`, `drawer`, `tabs`,
`dropdown`, `tooltip`, `toast`, `skeleton`, `badge`, `avatar`, `menu`, `timeline`)
plus its theme system. Class-only: no JavaScript ships to the browser, and we own
all behaviour (focus traps, keyboard navigation, roving tabindex).

**Why vendored.** daisyUI is normally an npm dependency, and this project has no
Node in its build (the only Node consent is Playwright, dev-only). Its Tailwind v4
entry point is `@plugin "./vendor/daisyui/index.js"`, which the Tailwind *standalone*
CLI evaluates without npm. Vendoring the package keeps the build a single binary
download and keeps the offline devcontainer working.

**The whole package is here on purpose** (3 MB, ~1.5 MB of it component CSS). The
plugin reads `imports.js` at build time to decide which components to emit, so
pruning `components/` would compile and then silently ship a stylesheet missing
whatever we pruned. The build output is what shrinks — one page using `btn`,
`card`, `badge` and `skeleton` produces **22 KB raw / 4.9 KB gzipped** of CSS.

**Do not edit anything in this directory.** It must stay byte-identical to the
published tarball, so an update is a diff of versions rather than a merge of local
patches. Our overrides live in `crates/web/style/theme.css`.

**How to update**

```sh
cd /tmp && rm -rf daisyui && mkdir daisyui && cd daisyui
npm pack daisyui@5.1.10            # or: curl -L $(npm view daisyui dist.tarball)
tar xzf daisyui-*.tgz
rm -rf /workspaces/MastodonReccom/web/vendor/daisyui
cp -a package /workspaces/MastodonReccom/web/vendor/daisyui
rm -f /workspaces/MastodonReccom/web/vendor/daisyui/README.md
```

Then update the version in this file, in `crates/web/style/app.css` if the plugin
path changed, and in the `[tools]` pin in `Trunk.toml` if the bundled Tailwind
version has to move with it.
