//! Generates the two browser artifacts the app cannot be built without.
//!
//! `crates/web-client/public/vendor/` (the ProseMirror bundle, the picker and the emoji
//! data file) and `crates/web-client/style/tailwind.css` (Tailwind + daisyUI) are derived: they
//! are produced from `js/`, `style/app.css`, `package.json` and `package-lock.json`,
//! which are the only things under version control. Nothing here is a source file, so
//! nothing here belongs in a commit.
//!
//! Why a build script and not a task in the README: `trunk` runs `cargo build`, so a
//! missing asset should be a build failure with a precise message, not a page that loads
//! and does nothing. The script is deliberately quiet when the outputs are already newer
//! than their sources, so an ordinary incremental build does not pay for it again.
//!
//! Requires Node (the devcontainer installs it as a feature) and a one-time
//! `npm install` in `web/`. If Node is absent and an asset is missing, this fails loudly
//! instead of producing a bundle that renders a composer with no editor.

use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let web = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/web-client lives inside web/")
        .to_path_buf();

    // Sources: the bundles' entry points, their dependencies, and the stylesheet.
    for watched in [
        "js/build.mjs",
        "js/css.mjs",
        "js/editor.mjs",
        "js/picker.mjs",
        "js/schema.mjs",
        "package.json",
        "package-lock.json",
        "crates/web/style/app.css",
    ] {
        println!("cargo:rerun-if-changed={watched}");
    }

    let outputs = [
        web.join("crates/web-client/public/vendor/editor.js"),
        web.join("crates/web-client/public/vendor/picker.js"),
        web.join("crates/web-client/public/vendor/emoji-data.json"),
        web.join("crates/web-client/style/tailwind.css"),
    ];

    if outputs.iter().all(|output| exists(output)) && !stale(&web, &outputs) {
        return;
    }

    let node_modules = web.join("node_modules");
    if !exists(&node_modules) {
        run(&web, "npm", &["ci", "--silent", "--no-audit", "--no-fund"]);
    }

    run(&web, "node", &["js/build.mjs"]);
    run(&web, "node", &["js/css.mjs"]);

    for output in &outputs {
        assert!(
            exists(output),
            "nodus: {} was not produced by the asset build",
            output.display()
        );
    }
}

fn exists(path: &Path) -> bool {
    path.exists()
}

/// True when any output is older than any source, or any output is missing.
fn stale(web: &Path, outputs: &[PathBuf]) -> bool {
    let Some(newest_source) = newest_mtime(&[
        web.join("js/editor.mjs"),
        web.join("js/picker.mjs"),
        web.join("js/schema.mjs"),
        web.join("js/build.mjs"),
        web.join("js/css.mjs"),
        web.join("crates/web/style/app.css"),
        web.join("package-lock.json"),
    ]) else {
        return true;
    };

    outputs.iter().any(|output| {
        std::fs::metadata(output)
            .and_then(|meta| meta.modified())
            .is_ok_and(|mtime| mtime < newest_source)
    })
}

fn newest_mtime(paths: &[PathBuf]) -> Option<std::time::SystemTime> {
    paths
        .iter()
        .filter_map(|path| {
            std::fs::metadata(path)
                .and_then(|meta| meta.modified())
                .ok()
        })
        .max()
}

fn run(web: &Path, program: &str, args: &[&str]) {
    let status = Command::new(program)
        .args(args)
        .current_dir(web)
        .status()
        .unwrap_or_else(|error| {
            panic!(
                "nodus: could not run `{program} {}` in {}: {error}\n\
                 The composer's bundles and the stylesheet are generated from `{}`; \
                 install Node and run `npm install` there once.",
                args.join(" "),
                web.display(),
                web.join("package.json").display()
            )
        });

    assert!(
        status.success(),
        "nodus: `{program} {}` failed with {status}",
        args.join(" ")
    );
}
