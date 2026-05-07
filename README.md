<div align="center">
  <img src="./docs/logo.svg" alt="duvis" width="180" height="auto" />
  <h1>Duvis</h1>
  <p align="center">
    <span><b>D</b>isk <b>U</b>sage <b>Vis</b>ualizer for both AI and humans</span>
  </p>
</div>

<hr />

<p align="center">
  <a href="https://crates.io/crates/duvis"><img src="https://img.shields.io/crates/v/duvis.svg?maxAge=1000" alt="crates.io"></a>
  <a href="https://crates.io/crates/duvis"><img src="https://img.shields.io/crates/d/duvis.svg" alt="downloads"></a>
  <a href="https://github.com/yamadashy/duvis/actions/workflows/ci.yml"><img src="https://github.com/yamadashy/duvis/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://codecov.io/gh/yamadashy/duvis"><img src="https://codecov.io/gh/yamadashy/duvis/graph/badge.svg?token=ASDJTR0FM7" alt="Codecov"></a>
  <a href="https://docs.rs/duvis"><img src="https://docs.rs/duvis/badge.svg" alt="docs.rs"></a>
  <a href="./LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"></a>
</p>

📊 `duvis` (pronounced **/ˈduːvɪs/** — `du` as in *disk usage*, `vis` as in *visualizer*) is a fast, read-only disk usage analyzer for **both AI agents and humans**. Point it at any directory and get a structured JSON tree (so an agent can reason about what's filling your disk) or an interactive browser treemap with sunburst and list views (so you can click through it yourself). Every entry is auto-tagged by category — `cache`, `build`, `log`, `vcs`, `media`, `ide` — so the answer to *"what's eating my disk?"* shows up at a glance. duvis only **shows** you the picture; deleting is your call to make with your own tools.

<p align="center">
  <img src="./docs/screenshots/treemap-dark.png" alt="duvis running locally at 127.0.0.1:7515, showing a treemap of ~/ghq with category-colored cells (cache in orange, build output in red, version control in green) and a sidebar with per-category totals" />
</p>

## 🌟 Features

- **AI-friendly**: Hierarchical JSON with size, category, and freshness — agents can reason about disk usage in one pass.
- **Human-friendly**: Built-in browser UI (React + d3) with treemap, sunburst, and list views, color-coded by category. Drill in by clicking.
- **Fast**: Parallel directory scanning powered by [rayon](https://github.com/rayon-rs/rayon).
- **Read-only by design**: duvis never deletes, never recommends deletions, never prints `rm` commands. Permission errors are silently skipped.
- **Real disk footprint**: On Unix, sizes are the bytes actually occupied on disk (`st_blocks × 512`, same as `du`'s default), so sparse VM images like OrbStack's `data.img.raw` show their real footprint instead of the multi-terabyte logical size. Windows falls back to apparent size for now.
- **Cross-platform**: macOS, Linux, Windows.

## Install

```sh
cargo install duvis
```

Or build from source:

```sh
git clone https://github.com/yamadashy/duvis.git
cd duvis
cargo install --path .
```

## Usage

```sh
# Tree view of the current directory
duvis

# Limit depth and show top N entries
duvis ~/projects --depth 2 --top 10

# Category-aware summary (cache / build / log / media / vcs / ide / other)
duvis ~/projects --analyze

# Structured JSON output (for AI agents and scripts)
duvis ~/projects --json

# Open browser UI with an interactive treemap
duvis ~/projects --ui
```

### Options

| Flag | Description |
| --- | --- |
| `-d, --depth <N>` | Maximum depth to display |
| `-n, --top <N>` | Show only the top N entries by size |
| `--json` | Output as JSON |
| `--analyze` | Show category-based summary with reclaimable size |
| `--ui` | Open browser UI with treemap visualization |
| `--port <PORT>` | Port for UI server (default: `7515`, [see below](#why-port-7515)). Falls back to a free port if busy. |
| `--sort <size\|name>` | Sort order (default: `size`) |
| `--reverse` | Reverse sort order |

`--json` / `--analyze` / `--ui` are mutually exclusive; pass at most one. With none, the default tree view is shown.

## Output examples

### Tree

```
project (438.9 MB)
├── target/  [build] 438.8 MB
├── .git/    [vcs]    54.7 KB
├── src/              24.5 KB
└── Cargo.toml        418 B
```

### Analyze

```
Total: 438.9 MB

Category Summary:
  build      438.9 MB  100%  1 items  (rebuildable)

Potentially reclaimable: 438.9 MB (cache + build + log)
```

### JSON

```json
{
  "name": "project",
  "size": 460195536,
  "size_human": "438.9 MB",
  "is_dir": true,
  "category": "build",
  "modified_days_ago": 0,
  "children": [
    { "name": "target", "size": 460091645, "category": "build", ... }
  ]
}
```

## Categories

`duvis` classifies entries into seven categories so you can quickly see what kind of data is filling your disk:

- `cache` — package and tool caches (`node_modules/`, `.cache/`, `__pycache__/`, `.cargo/`, ...)
- `build` — build artifacts (`target/`, `dist/`, `build/`, `.next/`, ...)
- `log` — log files (`*.log`, `logs/`, ...)
- `media` — images, video, audio
- `vcs` — version control metadata (`.git/`, ...)
- `ide` — IDE/editor metadata (`.idea/`, `.vscode/`, ...)
- `other` — everything else

Categories are assigned by directory or file name. Once a directory is classified
as anything other than `other`, **everything inside it inherits that category** —
because that directory is the natural delete unit (`rm -rf node_modules` removes
the whole subtree as one). The outermost named ancestor wins, so a project root
that happens to contain a giant `node_modules` is *not* itself classified as
`cache`.

## Why port 7515?

In the spirit of [Vite's `5173`](https://vite.dev/) (`SITE`/`VITE` written
with Roman numerals + leet — `V`=5, `I`=1, `T`=7, `E`=3), `duvis` defaults to
**`7515`**: drop `D` (it's basically a closed `O`), tilt `U` on its side and
it's a `7`, `V`=5 (Roman), `I`=1, `S`=5 (leet). Not assigned to anything in
the IANA registry, far enough from the `8080`/`3000`/`5173` clash zones,
and if it's still busy on your machine `duvis` quietly falls back to a free
OS-assigned port.

## License

[MIT](./LICENSE) © Kazuki Yamada
