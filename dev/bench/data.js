window.BENCHMARK_DATA = {
  "lastUpdate": 1778073943387,
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
      }
    ]
  }
}