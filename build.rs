// Materializes the bundled browser UI into Cargo's `OUT_DIR` so `src/ui.rs`
// can `include_str!` it. Two paths:
//
// - **Developer checkout** (`ui/` present): run `npm run build` so the
//   bundled UI is always in sync with `ui/src/`, copy to OUT_DIR.
// - **End-user install from crates.io** (`ui/` excluded from the tarball):
//   read `prebuilt/ui.html` (which Cargo.toml `include` ships in the
//   tarball) and copy that to OUT_DIR. No Node required.
//
// `prebuilt/ui.html` is committed and refreshed by `just ui-build-prebuilt`
// before publishing. Cargo build never writes to `prebuilt/` itself, so the
// working tree stays clean and `cargo publish` doesn't need `--allow-dirty`.
//
// We deliberately avoid writing the bundle into `src/` — generated artifacts
// inside the source tree confuse readers and `git status`.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=ui/src");
    println!("cargo:rerun-if-changed=ui/index.html");
    println!("cargo:rerun-if-changed=ui/package.json");
    println!("cargo:rerun-if-changed=ui/package-lock.json");
    println!("cargo:rerun-if-changed=ui/vite.config.ts");
    println!("cargo:rerun-if-changed=ui/tsconfig.json");
    println!("cargo:rerun-if-changed=prebuilt/ui.html");

    let out_dir: PathBuf = env::var_os("OUT_DIR")
        .expect("OUT_DIR is set by cargo for build scripts")
        .into();
    let dst = out_dir.join("ui.html");

    let ui_dir = Path::new("ui");
    let prebuilt = Path::new("prebuilt/ui.html");

    if ui_dir.join("package.json").exists() {
        // Dev / repo build: rebuild from sources so the binary always reflects
        // the current ui/src/ state. Does NOT update prebuilt/ — that's a
        // separate, explicit step (`just ui-build-prebuilt`) so the working
        // tree only changes when the developer means it to.
        if !ui_dir.join("node_modules").exists() {
            warn("installing ui/ dependencies (npm ci)...");
            npm(&["--prefix", "ui", "ci"]);
        }
        warn("building ui/ via vite...");
        npm(&["--prefix", "ui", "run", "build"]);

        let dist = ui_dir.join("dist/index.html");
        assert!(
            dist.exists(),
            "expected vite output at {} after `npm run build`",
            dist.display()
        );
        fs::copy(&dist, &dst).unwrap_or_else(|e| {
            panic!("failed to copy {} -> {}: {e}", dist.display(), dst.display())
        });
    } else if prebuilt.exists() {
        // End-user install: tarball ships prebuilt/ui.html via Cargo.toml
        // `include`. Just stage it for include_str!.
        fs::copy(prebuilt, &dst).unwrap_or_else(|e| {
            panic!(
                "failed to copy {} -> {}: {e}",
                prebuilt.display(),
                dst.display()
            )
        });
    } else {
        panic!(
            "neither ui/ source nor prebuilt/ui.html is present. \
             A duvis tarball must include the prebuilt UI."
        );
    }
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
