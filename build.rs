// Generates src/ui/index.html from the React app under ui/ at compile time.
//
// The Rust crate embeds that file via `include_str!` in src/ui.rs. Two paths:
//
// - **Developer checkout** (ui/ present): run `npm run build` so the bundled
//   UI is always in sync with ui/src/.
// - **End-user install from crates.io** (ui/ excluded from the tarball): the
//   pre-built src/ui/index.html ships inside the tarball via `include` in
//   Cargo.toml. We just verify it landed.

use std::path::Path;
use std::process::Command;

fn main() {
    // Cargo only re-runs build.rs when one of these inputs changes. Listing
    // the output file too means we re-run if a developer accidentally deletes
    // src/ui/index.html — otherwise the include_str! would fail next build.
    println!("cargo:rerun-if-changed=ui/src");
    println!("cargo:rerun-if-changed=ui/index.html");
    println!("cargo:rerun-if-changed=ui/package.json");
    println!("cargo:rerun-if-changed=ui/package-lock.json");
    println!("cargo:rerun-if-changed=ui/vite.config.ts");
    println!("cargo:rerun-if-changed=ui/tsconfig.json");
    println!("cargo:rerun-if-changed=ui/scripts/copy-to-rust.mjs");
    println!("cargo:rerun-if-changed=src/ui/index.html");

    let target = Path::new("src/ui/index.html");

    if !Path::new("ui/package.json").exists() {
        assert!(
            target.exists(),
            "src/ui/index.html is missing and ui/ is not present. \
             This crate's tarball must include the built UI."
        );
        return;
    }

    // First-run: install npm deps if node_modules is missing.
    if !Path::new("ui/node_modules").exists() {
        warn("installing ui/ dependencies (npm ci)...");
        npm(&["--prefix", "ui", "ci"]);
    }

    warn("building ui/ via vite...");
    npm(&["--prefix", "ui", "run", "build"]);
}

fn warn(msg: &str) {
    println!("cargo:warning={msg}");
}

fn npm(args: &[&str]) {
    // Windows resolves shims via PATHEXT, but Command::new doesn't, so we have
    // to spell out npm.cmd there.
    let cmd = if cfg!(windows) { "npm.cmd" } else { "npm" };
    let status = Command::new(cmd).args(args).status().unwrap_or_else(|e| {
        panic!("failed to spawn `{cmd}`: {e}. Install Node.js to build the UI.")
    });
    assert!(status.success(), "`{cmd} {}` failed", args.join(" "));
}
