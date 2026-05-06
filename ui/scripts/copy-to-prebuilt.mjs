// Copy the vite output into prebuilt/ui.html. Run via `just ui-build-prebuilt`
// before `cargo publish` so end users (who install via crates.io and don't
// have Node) get the latest UI bundle in the published tarball.
//
// Day-to-day cargo build does NOT use this script — build.rs runs vite and
// copies straight to OUT_DIR, leaving the working tree untouched.
import { copyFile, mkdir } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const src = resolve(here, "..", "dist", "index.html");
const dst = resolve(here, "..", "..", "prebuilt", "ui.html");

await mkdir(dirname(dst), { recursive: true });
await copyFile(src, dst);
console.log(`copied ${src} -> ${dst}`);
