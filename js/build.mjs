//! Builds the two bundles the composer mounts, plus the emoji data file they search.
//!
//! Run from `web`: `npm install && npm run build`. `crates/web-client/build.rs` runs this
//! automatically when an output is missing or older than its source, so the files are
//! generated rather than committed.

import { build } from "esbuild";
import { copyFile, mkdir, readFile, stat } from "node:fs/promises";
import { gzipSync } from "node:zlib";

const OUT = "crates/web/public/vendor";

const targets = [
  {
    entryPoints: ["js/editor.mjs"],
    outfile: `${OUT}/editor.js`,
    format: "iife",
  },
  {
    entryPoints: ["js/picker.mjs"],
    outfile: `${OUT}/picker.js`,
    format: "iife",
  },
];

await mkdir(OUT, { recursive: true });

// The picker's own data: 2 500-odd standard emoji with every shortcode. Copied from the
// npm package rather than fetched, so a build needs no CDN and the version is pinned by
// `package-lock.json`.
await copyFile(
  "node_modules/emoji-picker-element-data/en/emojibase/data.json",
  `${OUT}/emoji-data.json`,
);

for (const target of targets) {
  await build({
    ...target,
    bundle: true,
    minify: true,
    target: ["es2020"],
    legalComments: "none",
    logLevel: "info",
  });
  const bytes = await readFile(target.outfile);
  const gzipped = gzipSync(bytes, { level: 9 }).length;
  const { size } = await stat(target.outfile);
  console.log(
    `${target.outfile}: ${(size / 1024).toFixed(0)} KB raw, ${(gzipped / 1024).toFixed(0)} KB gzipped`,
  );
}
