//! Compiles `crates/web-shared/style/app.css` into the stylesheet `index.html` links.
//!
//! Tailwind and daisyUI are the npm packages in this directory, so their versions are
//! pinned by `package-lock.json` rather than by a vendored copy or a standalone binary.
//! The output is generated, not committed: `crates/web-client/build.rs` runs this when the
//! sheet is missing or older than `app.css`.

import { execFileSync } from "node:child_process";
import { existsSync } from "node:fs";
import { stat } from "node:fs/promises";
import { gzipSync } from "node:zlib";
import { readFileSync } from "node:fs";

const cli = "node_modules/.bin/tailwindcss";
const out = "crates/web-client/style/tailwind.css";

if (!existsSync(cli)) {
  console.warn(`nodus: ${cli} is missing; keep the committed ${out} or run npm install`);
  process.exit(0);
}

execFileSync(
  cli,
  ["-i", "crates/web-shared/style/app.css", "-o", out, "--minify"],
  { stdio: "inherit" },
);

const size = (await stat(out)).size;
const gzipped = gzipSync(readFileSync(out)).length;
console.log(`${out}: ${(size / 1024).toFixed(0)} KB raw, ${(gzipped / 1024).toFixed(0)} KB gzipped`);
