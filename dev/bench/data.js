window.BENCHMARK_DATA = {
  "lastUpdate": 1778164239733,
  "repoUrl": "https://github.com/yamadashy/duvis",
  "entries": {
    "duvis Performance": [
      {
        "commit": {
          "author": {
            "email": "koukun0120@gmail.com",
            "name": "Kazuki Yamada",
            "username": "yamadashy"
          },
          "committer": {
            "email": "koukun0120@gmail.com",
            "name": "Kazuki Yamada",
            "username": "yamadashy"
          },
          "distinct": true,
          "id": "8b7a62f0b47f5f36f51ef21b8f2d84be116f9f29",
          "message": "ci: add performance benchmark history workflow\n\nMirror the perf-benchmark-history pattern from yamadashy/repomix so we\ncan spot scanner regressions over time. Each push to main runs the\nrelease binary against a 50k-file synthetic fixture across\nUbuntu/macOS/Windows, takes the median of 20-30 timed runs, and\ngithub-action-benchmark publishes the series to gh-pages\n(`dev/bench/`).\n\nThe fixture is generated fresh per job (10 × 10 × 500 small files,\n3 levels deep). Median + IQR are recorded so CI can alert when a commit\ncrosses 130% of the previous best.\n\nThe workflow only triggers on `main` for now and ignores doc/UI-only\nchanges since they don't affect scan timing.\n\nCo-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-05-06T19:42:05+09:00",
          "tree_id": "47f163c89dbec4a615a964ecfe3281bebd899558",
          "url": "https://github.com/yamadashy/duvis/commit/8b7a62f0b47f5f36f51ef21b8f2d84be116f9f29"
        },
        "date": 1778064386200,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "duvis scan (50k files) [macOS]",
            "value": 85.66,
            "range": "±33.1",
            "unit": "ms",
            "extra": "Median of 30 runs\nQ1: 70.14ms, Q3: 103.24ms\nMin: 57.01ms, Max: 154.42ms"
          },
          {
            "name": "duvis scan (50k files) [Linux]",
            "value": 57.43,
            "range": "±0.87",
            "unit": "ms",
            "extra": "Median of 20 runs\nQ1: 57.11ms, Q3: 57.98ms\nMin: 56.44ms, Max: 58.65ms"
          },
          {
            "name": "duvis scan (50k files) [Windows]",
            "value": 454.59,
            "range": "±2.65",
            "unit": "ms",
            "extra": "Median of 20 runs\nQ1: 453.39ms, Q3: 456.04ms\nMin: 448.87ms, Max: 465.02ms"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "koukun0120@gmail.com",
            "name": "Kazuki Yamada",
            "username": "yamadashy"
          },
          "committer": {
            "email": "koukun0120@gmail.com",
            "name": "Kazuki Yamada",
            "username": "yamadashy"
          },
          "distinct": true,
          "id": "8182dfd621daa9a472b1550ffee04ebe176c8f1e",
          "message": "refactor(entry): collapse is_dir + children into EntryKind enum\n\nEarlier `Entry` carried both `is_dir: bool` and `children: Option<Vec<Entry>>`,\nencoding the same fact twice. Nothing prevented an inconsistent value\n(`is_dir: false` with `Some(children)`) from being constructed, and each\noutput backend chose a different field to dispatch on.\n\nReplace the two fields with `kind: EntryKind { File, Dir(Vec<Entry>) }`.\nNow \"is this a directory?\" and \"does it have children?\" are the same\nquestion at the type level. Add `Entry::file(...)` / `Entry::dir(...)`\nconstructors so the (kind, size) pair stays consistent — the dir\nconstructor sums the children's sizes for the caller.\n\n`Serialize` is implemented by hand to keep the v0.1.0 wire shape:\n`is_dir: bool` + optional `children: [...]`. The browser UI and any AI\nagent consuming `/data.json` see no change.\n\nCallers (scanner, output/{tree,json,analyze}, ui server tests) now use\n`entry.is_dir()` / `entry.children()` accessor methods. Tests added in\nentry.rs cover the dir-size invariant, the file-no-children invariant,\nand the wire-format round trip.\n\nResolves Codex review item #3 from the earlier post-3a9e94c audit.\n\nCo-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-05-06T21:30:49+09:00",
          "tree_id": "5dc51785ae6d5b9988c121fe6141eec60d88b526",
          "url": "https://github.com/yamadashy/duvis/commit/8182dfd621daa9a472b1550ffee04ebe176c8f1e"
        },
        "date": 1778070782725,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "duvis scan (50k files) [macOS]",
            "value": 129.73,
            "range": "±48",
            "unit": "ms",
            "extra": "Median of 30 runs\nQ1: 96.49ms, Q3: 144.49ms\nMin: 55.74ms, Max: 178.29ms"
          },
          {
            "name": "duvis scan (50k files) [Linux]",
            "value": 56.16,
            "range": "±0.95",
            "unit": "ms",
            "extra": "Median of 20 runs\nQ1: 55.66ms, Q3: 56.62ms\nMin: 54.8ms, Max: 57.11ms"
          },
          {
            "name": "duvis scan (50k files) [Windows]",
            "value": 719.42,
            "range": "±48.32",
            "unit": "ms",
            "extra": "Median of 20 runs\nQ1: 692.94ms, Q3: 741.26ms\nMin: 680.07ms, Max: 774.87ms"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "koukun0120@gmail.com",
            "name": "Kazuki Yamada",
            "username": "yamadashy"
          },
          "committer": {
            "email": "koukun0120@gmail.com",
            "name": "Kazuki Yamada",
            "username": "yamadashy"
          },
          "distinct": true,
          "id": "c7a2e38b0badd965c68e9712fab942f24685577a",
          "message": "refactor(build): write generated UI to OUT_DIR, ship prebuilt/ui.html\n\nEarlier `build.rs` wrote the bundled browser UI into `src/ui/index.html`\n(gitignored), and Cargo.toml's `include` list pulled it into the\npublished tarball. Two complaints, both raised by Codex:\n1. Generated artifacts inside `src/` confuse readers and `git status`.\n2. The gitignored output forced `cargo publish --allow-dirty`, weakening\n   the safety check that the tarball matches HEAD.\n\nRestructure:\n\n- `prebuilt/ui.html` is now committed and is the canonical bundle that\n  ships in the tarball (Cargo.toml `include` lists it).\n- `build.rs` writes only to `$OUT_DIR/ui.html`. Two paths:\n  * dev / repo build (`ui/` exists): runs `npm run build`, copies\n    `ui/dist/index.html` → OUT_DIR. `prebuilt/` is NOT touched, so the\n    working tree stays clean during routine cargo builds.\n  * end-user install from crates.io (`ui/` excluded from tarball):\n    copies `prebuilt/ui.html` → OUT_DIR. No Node required.\n- `src/ui.rs` reads via `include_str!(concat!(env!(\"OUT_DIR\"), \"/ui.html\"))`.\n- `prebuilt/ui.html` is refreshed by an explicit `just ui-build-prebuilt`\n  step, which is invoked automatically inside `just publish` /\n  `just publish-dry` so the published tarball always reflects the\n  current `ui/src/`.\n- `cargo publish` (and dry-run) no longer need `--allow-dirty`. The\n  publish recipe also auto-commits the `prebuilt/ui.html` refresh if\n  there's a diff, keeping the working tree clean.\n- `src/ui/` directory and its `.gitignore` removed, plus the\n  `/src/ui/index.html` line in the root `.gitignore`.\n- `ui/scripts/copy-to-rust.mjs` renamed to\n  `ui/scripts/copy-to-prebuilt.mjs` to reflect its new role.\n\nEnd-user `cargo install duvis` flow is unchanged: tarball contains\n`prebuilt/ui.html`, build.rs uses it, no Node ever invoked.\n\nResolves Codex review item #4 from the post-3a9e94c audit.\n\nCo-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-05-06T21:36:00+09:00",
          "tree_id": "ac8f58a73ba0b1b2a541baea7829b6401978e639",
          "url": "https://github.com/yamadashy/duvis/commit/c7a2e38b0badd965c68e9712fab942f24685577a"
        },
        "date": 1778071098592,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "duvis scan (50k files) [macOS]",
            "value": 105.68,
            "range": "±43",
            "unit": "ms",
            "extra": "Median of 30 runs\nQ1: 86.81ms, Q3: 129.81ms\nMin: 73.03ms, Max: 188.28ms"
          },
          {
            "name": "duvis scan (50k files) [Linux]",
            "value": 56.71,
            "range": "±0.93",
            "unit": "ms",
            "extra": "Median of 20 runs\nQ1: 56.46ms, Q3: 57.39ms\nMin: 56.03ms, Max: 59.95ms"
          },
          {
            "name": "duvis scan (50k files) [Windows]",
            "value": 470.19,
            "range": "±75.92",
            "unit": "ms",
            "extra": "Median of 20 runs\nQ1: 462.57ms, Q3: 538.49ms\nMin: 455.58ms, Max: 587.63ms"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "koukun0120@gmail.com",
            "name": "Kazuki Yamada",
            "username": "yamadashy"
          },
          "committer": {
            "email": "koukun0120@gmail.com",
            "name": "Kazuki Yamada",
            "username": "yamadashy"
          },
          "distinct": true,
          "id": "08a1cfe3b0338552e4442c71150c1410a9ffe7c4",
          "message": "build: gitignore prebuilt/ui.html, accept --allow-dirty on publish\n\nWalked back committing the prebuilt UI bundle. Tracking it in git would\nadd ~200KB on every UI change, bloating history for an artifact that's\ndeterministically derivable from `ui/src/`.\n\nprebuilt/ui.html is now gitignored and regenerated by `just ui-build-prebuilt`\nright before each publish. Cargo include still ships it, so end users\nwho `cargo install duvis` get the bundle without needing Node.\n\nTrade-off: `cargo publish` needs `--allow-dirty` again (the file is\nuntracked at publish time). The justfile publish/publish-dry recipes\ntake care of that — manual `cargo publish` outside the recipe would\nneed to remember the flag.\n\nEverything else from the OUT_DIR refactor stays: `prebuilt/ui.html`\nlocation is cleaner than `src/ui/index.html`, build.rs writes only to\n$OUT_DIR, and dev cargo build never touches the working tree.\n\nCo-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-05-06T22:21:02+09:00",
          "tree_id": "cb69376e06b02db9f1aa2b7409699f30712a73eb",
          "url": "https://github.com/yamadashy/duvis/commit/08a1cfe3b0338552e4442c71150c1410a9ffe7c4"
        },
        "date": 1778073943048,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "duvis scan (50k files) [macOS]",
            "value": 64.47,
            "range": "±42.31",
            "unit": "ms",
            "extra": "Median of 30 runs\nQ1: 52.04ms, Q3: 94.35ms\nMin: 50.72ms, Max: 194.94ms"
          },
          {
            "name": "duvis scan (50k files) [Linux]",
            "value": 56.18,
            "range": "±0.94",
            "unit": "ms",
            "extra": "Median of 20 runs\nQ1: 55.74ms, Q3: 56.68ms\nMin: 54.45ms, Max: 57.69ms"
          },
          {
            "name": "duvis scan (50k files) [Windows]",
            "value": 518.77,
            "range": "±3.36",
            "unit": "ms",
            "extra": "Median of 20 runs\nQ1: 516.53ms, Q3: 519.89ms\nMin: 512.37ms, Max: 527.43ms"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "koukun0120@gmail.com",
            "name": "Kazuki Yamada",
            "username": "yamadashy"
          },
          "committer": {
            "email": "koukun0120@gmail.com",
            "name": "Kazuki Yamada",
            "username": "yamadashy"
          },
          "distinct": true,
          "id": "48730822fe2fa03cf645d967bab964dd21177c96",
          "message": "fix(cli): exit silently on SIGPIPE instead of printing 'Broken pipe'\n\n`duvis ~/ghq --json | head` was surfacing\n`Error: Broken pipe (os error 32)` on stderr because Rust's runtime\ninstalls SIG_IGN for SIGPIPE, turning every dropped pipe write into an\nio::Error that bubbles up through anyhow.\n\nRestore SIGPIPE's default disposition (SIG_DFL) at startup on Unix so the\nprocess is killed by the signal — same behavior as `cat`, `du`, `grep`,\nripgrep, fd, and any other Unix CLI that streams to stdout. The exit\nstatus becomes 141 under `set -o pipefail`, which is the expected POSIX\nconvention. No-op on Windows.\n\nCo-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-05-06T23:10:14+09:00",
          "tree_id": "a2ea9503ac891f2d57d3b2a44ff548a049599a32",
          "url": "https://github.com/yamadashy/duvis/commit/48730822fe2fa03cf645d967bab964dd21177c96"
        },
        "date": 1778076755540,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "duvis scan (50k files) [macOS]",
            "value": 88.31,
            "range": "±30.24",
            "unit": "ms",
            "extra": "Median of 30 runs\nQ1: 70.35ms, Q3: 100.59ms\nMin: 48.45ms, Max: 153.17ms"
          },
          {
            "name": "duvis scan (50k files) [Linux]",
            "value": 46.39,
            "range": "±1.01",
            "unit": "ms",
            "extra": "Median of 20 runs\nQ1: 46.02ms, Q3: 47.03ms\nMin: 45.69ms, Max: 49.31ms"
          },
          {
            "name": "duvis scan (50k files) [Windows]",
            "value": 459.97,
            "range": "±26.08",
            "unit": "ms",
            "extra": "Median of 20 runs\nQ1: 457.18ms, Q3: 483.26ms\nMin: 455.37ms, Max: 510.62ms"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "koukun0120@gmail.com",
            "name": "Kazuki Yamada",
            "username": "yamadashy"
          },
          "committer": {
            "email": "koukun0120@gmail.com",
            "name": "Kazuki Yamada",
            "username": "yamadashy"
          },
          "distinct": true,
          "id": "4d52742fdbae875be5d52258f439479124cb84f1",
          "message": "refactor!: drop deletion-flavored language from CLI, UI, and categories\n\nduvis is read-only by design — but a lot of the original output and UI\ncopy was nudging the reader toward deletions (\"Potentially reclaimable\",\n\"Safe to delete\", \"Rebuildable\", \"safely deletable\", \"natural delete\nunit\"). Strip all of it. duvis describes what's there; what to do about\nit is the user's call.\n\nCLI / Rust:\n- Remove `Category::is_deletable()` and `Category::deletable_hint()`\n- `--analyze` no longer prints per-row hints or a \"Potentially\n  reclaimable\" total — just a per-category size summary\n- `--analyze` doc string reworded (\"Show a per-category size summary\")\n- `meta_block()` no longer ships `deletable_categories` to the UI\n- Rebuild snapshot to match the new analyze output\n\nUI:\n- Drop the \"Reclaimable\" stat block (replaced with a neutral \"Files\"\n  count)\n- Remove the \"Safe to delete\" / \"Rebuildable\" tag chips on the detail\n  panel and the corresponding \"Hint\" section\n- `CategoryMeta.tag` (\"safe\" | \"warn\") removed; `desc` is now a plain\n  factual one-liner (\"Package and tool caches\", \"Build artifacts\", ...)\n- `aggregate()` no longer computes `deletable`\n- Delete leftover CSS for the removed surfaces\n\nAlso fix a related miscategorization: `.ts` was being tagged as `media`\n(it's the MPEG transport-stream extension), but in practice every `.ts`\nfile on a developer's disk is TypeScript. Drop it from the media list.\n\nREADME:\n- Options table: \"reclaimable size\" → \"per-category size summary\"\n- Output examples: drop the \"(rebuildable)\" annotation and the\n  \"Potentially reclaimable\" line\n- Categories section: \"natural delete unit\" → \"natural grouping unit\",\n  drop the `rm -rf node_modules` example\n\nCo-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-05-07T21:33:53+09:00",
          "tree_id": "f90b7c9871346d5be8ddd82224e96a7b6d31c508",
          "url": "https://github.com/yamadashy/duvis/commit/4d52742fdbae875be5d52258f439479124cb84f1"
        },
        "date": 1778157381336,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "duvis scan (50k files) [macOS]",
            "value": 106.65,
            "range": "±18.75",
            "unit": "ms",
            "extra": "Median of 30 runs\nQ1: 95.99ms, Q3: 114.74ms\nMin: 80.04ms, Max: 187.76ms"
          },
          {
            "name": "duvis scan (50k files) [Linux]",
            "value": 45.7,
            "range": "±1.03",
            "unit": "ms",
            "extra": "Median of 20 runs\nQ1: 45.25ms, Q3: 46.28ms\nMin: 44.85ms, Max: 46.61ms"
          },
          {
            "name": "duvis scan (50k files) [Windows]",
            "value": 452.54,
            "range": "±10.9",
            "unit": "ms",
            "extra": "Median of 20 runs\nQ1: 451.15ms, Q3: 462.05ms\nMin: 445.81ms, Max: 481.29ms"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "koukun0120@gmail.com",
            "name": "Kazuki Yamada",
            "username": "yamadashy"
          },
          "committer": {
            "email": "koukun0120@gmail.com",
            "name": "Kazuki Yamada",
            "username": "yamadashy"
          },
          "distinct": true,
          "id": "f1a6e51eb80772ae202e361adf2f5f156aa51ff5",
          "message": "ci: publish to crates.io via Trusted Publishing instead of an API token\n\nAdd `.github/workflows/cargo-publish.yml`, mirroring the structure of\npdfvision's npm publish workflow but pointing at crates.io's OIDC-based\nTrusted Publishing (GA'd by crates.io in mid-2025).\n\nBehavior:\n- `workflow_dispatch` trigger with a `dry-run` input\n- fails early if Cargo.toml version is already on crates.io\n- runs fmt / clippy / cargo test / UI typecheck\n- builds prebuilt/ui.html so the published tarball ships the latest UI\n- mints a short-lived publish token via `rust-lang/crates-io-auth-action`\n  (no CARGO_REGISTRY_TOKEN secret in this repo)\n- on real (non-dry) publish, also creates the GitHub Release tag\n\nDrop the corresponding `just publish` recipe — local publishing was\nuseful when we only had API tokens, but mixing the two paths invites\n\"oops, I published from my laptop with the wrong branch checked out\"\nmistakes. Local `just publish-dry` stays for sanity-checking the\npackage contents without uploading.\n\nSetup still required (one-time, on crates.io dashboard):\n  duvis → Settings → Trusted Publishers → Add\n    Repository owner: yamadashy\n    Repository name:  duvis\n    Workflow filename: cargo-publish.yml\n    Environment:      (leave blank)\n\nCo-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-05-07T22:11:54+09:00",
          "tree_id": "80502b65f7aa46c51937a6348664f69d46e55a9f",
          "url": "https://github.com/yamadashy/duvis/commit/f1a6e51eb80772ae202e361adf2f5f156aa51ff5"
        },
        "date": 1778159713274,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "duvis scan (50k files) [macOS]",
            "value": 49.79,
            "range": "±2.95",
            "unit": "ms",
            "extra": "Median of 30 runs\nQ1: 48.99ms, Q3: 51.94ms\nMin: 48.21ms, Max: 86.29ms"
          },
          {
            "name": "duvis scan (50k files) [Linux]",
            "value": 55.85,
            "range": "±1.82",
            "unit": "ms",
            "extra": "Median of 20 runs\nQ1: 55.08ms, Q3: 56.91ms\nMin: 54.56ms, Max: 58.84ms"
          },
          {
            "name": "duvis scan (50k files) [Windows]",
            "value": 528.9,
            "range": "±10.01",
            "unit": "ms",
            "extra": "Median of 20 runs\nQ1: 525.79ms, Q3: 535.8ms\nMin: 519.9ms, Max: 637.23ms"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "koukun0120@gmail.com",
            "name": "Kazuki Yamada",
            "username": "yamadashy"
          },
          "committer": {
            "email": "koukun0120@gmail.com",
            "name": "Kazuki Yamada",
            "username": "yamadashy"
          },
          "distinct": true,
          "id": "04602c5733a10b1c98554f6bb2e3b78ebb1ef430",
          "message": "ci(publish): split into verify/publish/release jobs with least-privilege scopes\n\nCodex review of the original single-job cargo-publish.yml flagged two\nreal holes and a few smaller things. Address them:\n\nMust-fix:\n- `workflow_dispatch` lets you pick any ref from the Actions UI, and\n  crates.io's trusted-publisher entry has no Environment policy to\n  restrict branch. A feature branch dispatch could therefore publish a\n  modified workflow. Gate the publish + release jobs to\n  `github.ref == 'refs/heads/main'`.\n\nShould-fix:\n- The original job carried `contents: write` for the entire publish run\n  (needed only by `gh release create`), so any third-party action in the\n  same job — `setup-node`, `rust-toolchain`, `rust-cache` — implicitly\n  inherited write access alongside the live crates.io token. Split into\n  three jobs: verify (read-only), publish (id-token + read), release\n  (contents:write only). The crates.io token never coexists with\n  contents:write in the same job context.\n- The dry-run path was minting an OIDC token even though\n  `cargo publish --dry-run` doesn't upload. Move dry-run into the verify\n  job, which never calls `crates-io-auth-action`. Real publishes are the\n  only path that mints a token.\n\nNice-to-have:\n- `concurrency: cargo-publish` blocks racing dispatches.\n- `set -euo pipefail` on bash steps so failed pipes don't slip through.\n\nWhat's NOT changed (deliberate, with rationale):\n- Third-party actions are still tag-pinned (`@v4`, `@v2`, `@v1`,\n  `@stable`) rather than SHA-pinned. Tag pinning is the industry\n  baseline; SHA pinning is stricter but adds maintenance churn. The\n  actions used (actions/*, rust-lang/*, dtolnay/rust-toolchain,\n  Swatinem/rust-cache) are de-facto standards with active maintainers,\n  so the trade-off favors readability for a solo-maintained crate.\n- `cargo semver-checks` is not added here; that's a feature decision\n  (do we want semver enforcement at all?) and belongs in its own change.\n\nBranch protection on main + a GitHub Actions Environment named \"release\"\n(with deployment-branch policy) would close the remaining gap, but both\nrequire dashboard setup outside this workflow file.\n\nCo-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-05-07T22:27:34+09:00",
          "tree_id": "11db4038fcff7a14bda58b9b38bd408d5469ca52",
          "url": "https://github.com/yamadashy/duvis/commit/04602c5733a10b1c98554f6bb2e3b78ebb1ef430"
        },
        "date": 1778160603029,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "duvis scan (50k files) [macOS]",
            "value": 64.86,
            "range": "±24.21",
            "unit": "ms",
            "extra": "Median of 30 runs\nQ1: 54.59ms, Q3: 78.79ms\nMin: 50.92ms, Max: 117.68ms"
          },
          {
            "name": "duvis scan (50k files) [Linux]",
            "value": 72.45,
            "range": "±0.64",
            "unit": "ms",
            "extra": "Median of 20 runs\nQ1: 72.21ms, Q3: 72.84ms\nMin: 66.93ms, Max: 73.46ms"
          },
          {
            "name": "duvis scan (50k files) [Windows]",
            "value": 517.69,
            "range": "±4.04",
            "unit": "ms",
            "extra": "Median of 20 runs\nQ1: 514.93ms, Q3: 518.97ms\nMin: 505.67ms, Max: 538.83ms"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "koukun0120@gmail.com",
            "name": "Kazuki Yamada",
            "username": "yamadashy"
          },
          "committer": {
            "email": "koukun0120@gmail.com",
            "name": "Kazuki Yamada",
            "username": "yamadashy"
          },
          "distinct": true,
          "id": "8066ef38ed747e81b177ce0cde31f008f1fc5bbc",
          "message": "chore: bump version to 0.1.1\n\nPatch release covering everything since 0.1.0:\n\nBug fixes\n- Restore default SIGPIPE behavior so `duvis ... | head` exits silently\n  instead of printing \"Error: Broken pipe (os error 32)\".\n- Drop `.ts` from the media-extension list — TypeScript files vastly\n  outnumber MPEG transport-stream files in real codebases, and\n  classifying `index.ts` as `media` was a daily annoyance.\n\nRead-only stance, fully implemented\n- Remove deletion-flavored language from the CLI: `--analyze` no longer\n  prints `(rebuildable)`, `(safely deletable)`, or \"Potentially\n  reclaimable: …\".\n- Remove deletion-flavored language from the browser UI: the\n  \"Reclaimable\" stat block, \"Safe to delete\" / \"Rebuildable\" tag chips,\n  and the per-category \"Hint\" section are gone.\n- Add a deliberately disabled \"Move to trash\" button in the detail panel\n  that, on hover, explains duvis is read-only by design and points the\n  user at their OS-native delete tools.\n\nDocs\n- Pronunciation note (`/ˈduːvɪs/`).\n- Vite-style intro: tagline, scannable feature list, then prose.\n- \"How sizes are measured\" section (st_blocks vs apparent, sparse files,\n  Windows fallback) split out of the lead.\n\nCo-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-05-07T22:33:32+09:00",
          "tree_id": "91d281d8aa37b096a58bf13595b004065e689146",
          "url": "https://github.com/yamadashy/duvis/commit/8066ef38ed747e81b177ce0cde31f008f1fc5bbc"
        },
        "date": 1778160995080,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "duvis scan (50k files) [macOS]",
            "value": 85.76,
            "range": "±51.51",
            "unit": "ms",
            "extra": "Median of 30 runs\nQ1: 64.24ms, Q3: 115.75ms\nMin: 52.32ms, Max: 151.94ms"
          },
          {
            "name": "duvis scan (50k files) [Linux]",
            "value": 56.63,
            "range": "±0.56",
            "unit": "ms",
            "extra": "Median of 20 runs\nQ1: 56.43ms, Q3: 56.99ms\nMin: 56ms, Max: 58.06ms"
          },
          {
            "name": "duvis scan (50k files) [Windows]",
            "value": 507.22,
            "range": "±6.19",
            "unit": "ms",
            "extra": "Median of 20 runs\nQ1: 504.93ms, Q3: 511.12ms\nMin: 500.41ms, Max: 538.89ms"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "koukun0120@gmail.com",
            "name": "Kazuki Yamada",
            "username": "yamadashy"
          },
          "committer": {
            "email": "koukun0120@gmail.com",
            "name": "Kazuki Yamada",
            "username": "yamadashy"
          },
          "distinct": true,
          "id": "533c2fd0eb816723863ade311cba87132513e8c7",
          "message": "ci(publish): also gate jobs through GitHub Actions Environment 'release'\n\nAdds `environment: release` to the publish + release jobs so the\ndeployment-branch policy configured in repo settings (main only)\nparticipates in the gating, on top of the existing\n`if: github.ref == 'refs/heads/main'` workflow-file check.\n\nThree independent gates now all have to agree to publish:\n  1. GitHub Actions Environment policy refuses to schedule the job from\n     a non-main ref.\n  2. `if: github.ref == 'refs/heads/main'` in the workflow file catches\n     misconfigurations of (1).\n  3. crates.io's trusted publisher entry pins environment=release, so a\n     workflow that drops `environment: release` can't get an OIDC token.\n\nDefense in depth: any one of these can be removed by mistake without\nopening a publish path.\n\nSetup left for the user (one-time, on crates.io dashboard):\n  duvis → Settings → Trusted Publishers → Edit/Re-add\n    Environment: release   (was blank)\n\nCo-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-05-07T22:40:02+09:00",
          "tree_id": "004465bc9709480ef4cb4bc15f0b2715291de7c1",
          "url": "https://github.com/yamadashy/duvis/commit/533c2fd0eb816723863ade311cba87132513e8c7"
        },
        "date": 1778161469880,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "duvis scan (50k files) [macOS]",
            "value": 85.49,
            "range": "±39.37",
            "unit": "ms",
            "extra": "Median of 30 runs\nQ1: 64.36ms, Q3: 103.73ms\nMin: 54.2ms, Max: 141.37ms"
          },
          {
            "name": "duvis scan (50k files) [Linux]",
            "value": 56.66,
            "range": "±0.79",
            "unit": "ms",
            "extra": "Median of 20 runs\nQ1: 56.44ms, Q3: 57.23ms\nMin: 56.25ms, Max: 60.04ms"
          },
          {
            "name": "duvis scan (50k files) [Windows]",
            "value": 345.77,
            "range": "±3.21",
            "unit": "ms",
            "extra": "Median of 20 runs\nQ1: 344.39ms, Q3: 347.6ms\nMin: 343.02ms, Max: 366.49ms"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "koukun0120@gmail.com",
            "name": "Kazuki Yamada",
            "username": "yamadashy"
          },
          "committer": {
            "email": "koukun0120@gmail.com",
            "name": "Kazuki Yamada",
            "username": "yamadashy"
          },
          "distinct": true,
          "id": "5ef9c39f37fdb2e178567930ce1759fb46af95d4",
          "message": "feat(cli): add --version / -V flag\n\nclap's `version` attribute pulls the version string from CARGO_PKG_VERSION\nat build time, so `duvis --version` (and the conventional `-V` short form)\nnow print `duvis <version>` instead of erroring out with \"unexpected\nargument '--version' found\". Standard Unix CLI hygiene that v0.1.0/0.1.1\nshipped without.\n\nCo-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-05-07T22:49:15+09:00",
          "tree_id": "578f8c5c8fde73f47ed6b8af592b87a09496146e",
          "url": "https://github.com/yamadashy/duvis/commit/5ef9c39f37fdb2e178567930ce1759fb46af95d4"
        },
        "date": 1778161893828,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "duvis scan (50k files) [macOS]",
            "value": 88.49,
            "range": "±20.53",
            "unit": "ms",
            "extra": "Median of 30 runs\nQ1: 71.59ms, Q3: 92.12ms\nMin: 49.06ms, Max: 117.32ms"
          },
          {
            "name": "duvis scan (50k files) [Linux]",
            "value": 39.9,
            "range": "±0.55",
            "unit": "ms",
            "extra": "Median of 20 runs\nQ1: 39.78ms, Q3: 40.33ms\nMin: 39.46ms, Max: 48.07ms"
          },
          {
            "name": "duvis scan (50k files) [Windows]",
            "value": 516.45,
            "range": "±5.82",
            "unit": "ms",
            "extra": "Median of 20 runs\nQ1: 513.03ms, Q3: 518.85ms\nMin: 509.84ms, Max: 540.58ms"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "koukun0120@gmail.com",
            "name": "Kazuki Yamada",
            "username": "yamadashy"
          },
          "committer": {
            "email": "koukun0120@gmail.com",
            "name": "Kazuki Yamada",
            "username": "yamadashy"
          },
          "distinct": true,
          "id": "4d433b6202ea1838315afe38c2e97dbf7a06922a",
          "message": "docs(cli): expand --help into AI-agent-readable sections\n\nThe previous --help was a flat list of one-line descriptions. Repomix's\nhelp is the bar to clear: an agent reading it cold should be able to\ndrive the tool without ever opening the README. Mirror that density.\n\nChanges:\n- Group flags under \"Output Format\" / \"Display Options\" / \"UI Server\n  Options\" headings via clap's `help_heading`. Output formats appear\n  first because they're the primary axis of choice.\n- Long-form per-flag descriptions (one paragraph each on `--help`,\n  one line each on `-h`) covering: what it does, how it interacts with\n  related flags, gotchas (e.g. --top selects by size regardless of\n  --sort), and units / value names where relevant.\n- `long_about` on the command itself states the tool's purpose, the\n  three output modes, and the read-only stance up front.\n- `after_help` carries an EXAMPLES block with five copy-paste-ready\n  invocations, including a `--json | jq` recipe for agent pipelines.\n\nVisible to a first-time agent reader on `-h` (~30 lines, scannable),\nor on `--help` (paragraphs per flag, the full briefing).\n\nCo-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-05-07T22:55:28+09:00",
          "tree_id": "327091ae80bb21416500529d5908a06360dba0a3",
          "url": "https://github.com/yamadashy/duvis/commit/4d433b6202ea1838315afe38c2e97dbf7a06922a"
        },
        "date": 1778162338647,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "duvis scan (50k files) [macOS]",
            "value": 129.71,
            "range": "±42.29",
            "unit": "ms",
            "extra": "Median of 30 runs\nQ1: 100.8ms, Q3: 143.09ms\nMin: 66.5ms, Max: 193.81ms"
          },
          {
            "name": "duvis scan (50k files) [Linux]",
            "value": 57.27,
            "range": "±1.8",
            "unit": "ms",
            "extra": "Median of 20 runs\nQ1: 56.75ms, Q3: 58.55ms\nMin: 56.06ms, Max: 62.64ms"
          },
          {
            "name": "duvis scan (50k files) [Windows]",
            "value": 510.25,
            "range": "±5.09",
            "unit": "ms",
            "extra": "Median of 20 runs\nQ1: 508.01ms, Q3: 513.1ms\nMin: 504.99ms, Max: 559.97ms"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "koukun0120@gmail.com",
            "name": "Kazuki Yamada",
            "username": "yamadashy"
          },
          "committer": {
            "email": "koukun0120@gmail.com",
            "name": "Kazuki Yamada",
            "username": "yamadashy"
          },
          "distinct": true,
          "id": "e8371e2f22d8109db5f27a039996624a86067502",
          "message": "docs(cli): make -h and --help produce the same detailed output\n\nThe previous version exploited clap's two-tier help (short doc on -h,\nlong doc on --help), but the user prefers a single canonical help\noutput. Drop `long_about` and collapse every per-flag doc comment into\na single paragraph (no blank `///` line) so clap can't split short and\nlong.\n\nResult: typing `duvis -h` and `duvis --help` now produces identical\noutput — the full briefing every time. Sectioning, value names, and\nEXAMPLES block are unchanged.\n\nCo-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-05-07T23:19:08+09:00",
          "tree_id": "51aba1f83561b10cc8755bd2cfee9f403e057ffa",
          "url": "https://github.com/yamadashy/duvis/commit/e8371e2f22d8109db5f27a039996624a86067502"
        },
        "date": 1778163688297,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "duvis scan (50k files) [macOS]",
            "value": 112.01,
            "range": "±31.64",
            "unit": "ms",
            "extra": "Median of 30 runs\nQ1: 89.05ms, Q3: 120.69ms\nMin: 77.44ms, Max: 184.97ms"
          },
          {
            "name": "duvis scan (50k files) [Linux]",
            "value": 58.84,
            "range": "±3.63",
            "unit": "ms",
            "extra": "Median of 20 runs\nQ1: 57.25ms, Q3: 60.88ms\nMin: 56.1ms, Max: 72ms"
          },
          {
            "name": "duvis scan (50k files) [Windows]",
            "value": 461.84,
            "range": "±21.23",
            "unit": "ms",
            "extra": "Median of 20 runs\nQ1: 457.75ms, Q3: 478.98ms\nMin: 455.91ms, Max: 682.03ms"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "koukun0120@gmail.com",
            "name": "Kazuki Yamada",
            "username": "yamadashy"
          },
          "committer": {
            "email": "koukun0120@gmail.com",
            "name": "Kazuki Yamada",
            "username": "yamadashy"
          },
          "distinct": true,
          "id": "50775f7832fef7616c2d01bd4525406b6b907a64",
          "message": "docs(cli): document why --help is written so densely\n\nAdd a top-of-file editing note explaining that `--help` is a\nfirst-class deliverable for AI agents (not just humans), so future\nedits should keep the same density: precise per-flag behavior, explicit\ngotchas (e.g. --top selects by size regardless of --sort), single\nparagraph per `///` block to keep `-h` and `--help` identical, and a\nworking `--json | jq` example in `after_help`.\n\nWithout this note, a future me — or another contributor — might trim\ndescriptions down to one-liners thinking they're being concise, and\nsilently degrade the agent UX.\n\nCo-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-05-07T23:21:27+09:00",
          "tree_id": "16e8126c14356bcda8168111e69152cfbf28e653",
          "url": "https://github.com/yamadashy/duvis/commit/50775f7832fef7616c2d01bd4525406b6b907a64"
        },
        "date": 1778163916823,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "duvis scan (50k files) [macOS]",
            "value": 149.66,
            "range": "±36.87",
            "unit": "ms",
            "extra": "Median of 30 runs\nQ1: 128.92ms, Q3: 165.79ms\nMin: 75.55ms, Max: 216.62ms"
          },
          {
            "name": "duvis scan (50k files) [Linux]",
            "value": 55.82,
            "range": "±0.73",
            "unit": "ms",
            "extra": "Median of 20 runs\nQ1: 55.44ms, Q3: 56.17ms\nMin: 54.82ms, Max: 56.8ms"
          },
          {
            "name": "duvis scan (50k files) [Windows]",
            "value": 525.64,
            "range": "±68.37",
            "unit": "ms",
            "extra": "Median of 20 runs\nQ1: 519.43ms, Q3: 587.8ms\nMin: 507.91ms, Max: 632.8ms"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "koukun0120@gmail.com",
            "name": "Kazuki Yamada",
            "username": "yamadashy"
          },
          "committer": {
            "email": "koukun0120@gmail.com",
            "name": "Kazuki Yamada",
            "username": "yamadashy"
          },
          "distinct": true,
          "id": "009e9e11e0c943240d1983da5bd93473701eaa8a",
          "message": "docs(cli): tighten --help based on agent dogfooding (Codex review)\n\nAsked Codex (acting as an AI agent) to drive duvis using only the\ninformation in --help, then report every gap. Specific surprises it\nhit, now addressed in the help text:\n\n- `size` field semantics weren't documented. Codex saw a 1-byte file\n  report 4096 and a sparse file report 0, but had no way to know it's\n  `st_blocks * 512` (allocated disk bytes). Now stated in the --json\n  description.\n- `children` array conditions weren't documented. Codex didn't know\n  that files never carry it, and that depth-limited or --top-trimmed\n  directories silently omit it. Now stated.\n- --analyze silently ignores --depth / --top / --sort / --reverse —\n  Codex tried `duvis . --analyze --depth 1 --top 1` and got the full\n  summary regardless. Now stated explicitly on each affected flag and\n  on --analyze itself.\n- --analyze's `item count` is the number of category roots in each\n  bucket (e.g. one `target/` = 1 build item), not the file count\n  inside categorized directories. Now stated.\n- PATH accepted files even though --help said \"directory\". Now says\n  \"file or directory\".\n- Permission-denied paths are skipped silently with a stderr warning\n  and exit 0. An agent watching exit codes wouldn't know to check\n  stderr. Now stated on the PATH description.\n- Symlink behavior was vague — actually they appear as leaf entries\n  reporting the symlink's own disk usage, not the target's. Now\n  stated.\n- --ui's display flags also being ignored wasn't called out. Now\n  stated.\n\nAlso dropped agent-distracting prose (\"feeding into MCP servers\",\n\"persisting as a snapshot\", \"see the README for why this number\") in\nfavor of the schema/limits/behavior info Codex actually needed.\n\nExamples block: replaced `duvis .` (would produce ~20k lines on a\nproject root, agents would copy verbatim) with depth-limited variants.\n\nEditing-note checklist updated with the four checks Codex's review\nrevealed: schema completeness, default output size, ignored flag\ninteractions, stderr/non-zero exit behaviors.\n\nCo-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-05-07T23:28:26+09:00",
          "tree_id": "c430b0f7105a0ddadcdbba6e0273fc47f8ab299f",
          "url": "https://github.com/yamadashy/duvis/commit/009e9e11e0c943240d1983da5bd93473701eaa8a"
        },
        "date": 1778164239280,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "duvis scan (50k files) [macOS]",
            "value": 109.01,
            "range": "±39.19",
            "unit": "ms",
            "extra": "Median of 30 runs\nQ1: 81.43ms, Q3: 120.61ms\nMin: 50.66ms, Max: 165.95ms"
          },
          {
            "name": "duvis scan (50k files) [Linux]",
            "value": 72.98,
            "range": "±1.3",
            "unit": "ms",
            "extra": "Median of 20 runs\nQ1: 72.56ms, Q3: 73.86ms\nMin: 65.51ms, Max: 89.65ms"
          },
          {
            "name": "duvis scan (50k files) [Windows]",
            "value": 515.47,
            "range": "±4.11",
            "unit": "ms",
            "extra": "Median of 20 runs\nQ1: 513.97ms, Q3: 518.09ms\nMin: 509.58ms, Max: 563.73ms"
          }
        ]
      }
    ]
  }
}