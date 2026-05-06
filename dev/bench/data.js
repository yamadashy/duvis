window.BENCHMARK_DATA = {
  "lastUpdate": 1778064386611,
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
      }
    ]
  }
}