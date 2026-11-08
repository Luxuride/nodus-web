//! Builds the two vendored bundles the composer mounts.
//!
//! Run from `web/js`: `npm install && npm run build`. The output is committed, so the Rust
//! build and the deployment never need npm — the same reason `Cargo.lock` is committed.

import { build } from "esbuild";
import { mkdir, readFile, stat } from "node:fs/promises";
import { gzipSync } from "node:zlib";

const OUT = "../crates/web/public/vendor";

const targets = [
  {
    entryPoints: ["editor.mjs"],
    outfile: `${OUT}/editor.js`,
    format: "iife",
  },
  {
    entryPoints: ["picker.mjs"],
    outfile: `${OUT}/picker.js`,
    format: "iife",
  },
];

await mkdir(OUT, { recursive: true });

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
