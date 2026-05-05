// Copy the built single-file UI into the Rust source tree so it gets embedded
// via include_str! during cargo build. The Rust crate doesn't depend on Node
// at install time -- we just commit the produced file.
import { copyFile, mkdir } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const src = resolve(here, "..", "dist", "index.html");
const dst = resolve(here, "..", "..", "src", "ui", "index.html");

await mkdir(dirname(dst), { recursive: true });
await copyFile(src, dst);
console.log(`copied ${src} -> ${dst}`);
