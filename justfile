# duvis dev tasks. Run `just` to see the list.
# https://just.systems  (`brew install just`  or  `cargo install just`)

set shell := ["bash", "-cu"]

# List available recipes.
default:
    @just --list

# ----- daily -----

# Run duvis against PATH (default `.`) with the browser UI.
dev path=".":
    cargo run --release -- {{path}} --ui

# Release build of the duvis binary (target/release/duvis).
build:
    cargo build --release

# Run the Rust unit tests.
test:
    cargo test --lib

# Run all tests (unit + integration).
test-all:
    cargo test

# Test coverage summary in the terminal. Requires `cargo install cargo-llvm-cov`.
coverage:
    cargo llvm-cov --summary-only

# Browseable HTML coverage report at target/llvm-cov/html/index.html.
coverage-html:
    cargo llvm-cov --html
    @echo "open target/llvm-cov/html/index.html"

# ----- UI iteration -----

# Vite dev server for iterating on ui/src without going through Rust.
ui-dev:
    cd ui && npm run dev

# Force a fresh UI bundle build (`cargo build` triggers this automatically).
ui-build:
    cd ui && npm run build

# Refresh the committed prebuilt/ui.html bundle. Run before `just publish`
# so end users (who install via crates.io and lack Node) get the latest UI.
ui-build-prebuilt:
    cd ui && npm run build:prebuilt

# Type-check the React UI.
typecheck:
    cd ui && npm run typecheck

# ----- formatting + lint -----

# Auto-format both Rust and TypeScript sources.
fmt:
    cargo fmt
    cd ui && npm run format

# Match CI: format check + clippy + typecheck.
lint:
    cargo fmt --all -- --check
    cargo clippy --all-targets -- -D warnings
    cd ui && npm run typecheck

# Run everything CI runs, locally.
check: lint test
    @echo "all checks passed"

# ----- release -----

# List the files that would ship to crates.io.
package:
    cargo package --list

# Verify a publish would succeed without uploading. Refreshes the gitignored
# prebuilt/ui.html first so the published tarball reflects the latest UI.
# `--allow-dirty` is required because prebuilt/ui.html is intentionally
# gitignored (generated artifacts shouldn't bloat git history).
publish-dry:
    just ui-build-prebuilt
    cargo publish --dry-run --allow-dirty

# Publish to crates.io + git tag + GitHub release (uses Cargo.toml version).
publish:
    #!/usr/bin/env bash
    set -euo pipefail
    VERSION=$(awk -F'"' '/^version =/ { print $2; exit }' Cargo.toml)
    if [[ -z "$VERSION" ]]; then
      echo "could not detect version in Cargo.toml" >&2
      exit 1
    fi
    echo "publishing duvis v$VERSION"
    just ui-build-prebuilt
    cargo publish --allow-dirty
    git tag "v$VERSION"
    git push origin "v$VERSION"
    gh release create "v$VERSION" --generate-notes

# ----- housekeeping -----

# Remove all build artifacts (Rust + UI). prebuilt/ui.html stays — it's
# tracked in git and refreshed via `just ui-build-prebuilt`.
clean:
    cargo clean
    rm -rf ui/node_modules ui/dist
