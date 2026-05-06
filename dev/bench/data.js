window.BENCHMARK_DATA = {
  "lastUpdate": 1778070783607,
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
      }
    ]
  }
}