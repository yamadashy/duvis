// Benchmark the `duvis` binary by timing N end-to-end scan runs against a
// synthetic fixture. Mirrors repomix's perf-benchmark-history shape so
// `github-action-benchmark` can feed the results into gh-pages charts.
//
// Inputs:
//   argv[2]:           path to the repo root (workspace dir)
//   env BENCH_RUNS:    number of measurement iterations (default 20)
//
// Output:
//   $RUNNER_TEMP/bench-result.json  in `customSmallerIsBetter` format
//   stdout:                         "<os>: median=<n>ms (±<iqr>ms)"

import { execFileSync } from 'node:child_process';
import { mkdirSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, sep } from 'node:path';

const dir = process.argv[2];
if (!dir) {
  console.error('Usage: node bench-run.mjs <workspace-dir>');
  process.exit(2);
}

const runs = Number(process.env.BENCH_RUNS) || 20;
const bin = join(dir, 'target', 'release', process.platform === 'win32' ? 'duvis.exe' : 'duvis');

// Build a synthetic fixture: 10 × 10 × 500 ≈ 50,000 small files arranged
// 3-deep. Big enough to amortize startup, small enough to keep CI runs
// under a few minutes. Created fresh per job so caches are warm but
// content is identical across runs of the same OS.
const fixture = join(tmpdir(), 'duvis-bench-fixture');
mkdirSync(fixture, { recursive: true });
for (let i = 0; i < 10; i++) {
  for (let j = 0; j < 10; j++) {
    const sub = join(fixture, `dir-${i}`, `sub-${j}`);
    mkdirSync(sub, { recursive: true });
    for (let k = 0; k < 500; k++) {
      writeFileSync(join(sub, `file-${k}.txt`), `payload ${i}/${j}/${k}\n`);
    }
  }
}

// Warm-up runs so the OS page cache and JIT-y bits are stable.
for (let i = 0; i < 3; i++) {
  try {
    execFileSync(bin, [fixture, '--max-depth', '1'], { stdio: 'ignore' });
  } catch {}
}

// Measurement runs.
const times = [];
for (let i = 0; i < runs; i++) {
  try {
    const start = process.hrtime.bigint();
    execFileSync(bin, [fixture, '--max-depth', '1'], { stdio: 'ignore' });
    const elapsedNs = process.hrtime.bigint() - start;
    times.push(Number(elapsedNs) / 1e6); // ms
  } catch (e) {
    console.error(`Run ${i + 1}/${runs} failed: ${e.message}`);
  }
}

if (times.length === 0) {
  console.error('All benchmark runs failed');
  process.exit(1);
}

times.sort((a, b) => a - b);
const median = times[Math.floor(times.length / 2)];
const q1 = times[Math.floor(times.length * 0.25)];
const q3 = times[Math.floor(times.length * 0.75)];
const iqr = q3 - q1;

const osName = process.env.RUNNER_OS || process.platform;
const round = (n) => Math.round(n * 100) / 100;

const result = [
  {
    name: `duvis scan (50k files) [${osName}]`,
    unit: 'ms',
    value: round(median),
    range: `±${round(iqr)}`,
    extra: [
      `Median of ${times.length} runs`,
      `Q1: ${round(q1)}ms, Q3: ${round(q3)}ms`,
      `Min: ${round(times[0])}ms, Max: ${round(times[times.length - 1])}ms`,
    ].join('\n'),
  },
];

const outDir = process.env.RUNNER_TEMP || tmpdir();
writeFileSync(join(outDir, 'bench-result.json'), JSON.stringify(result, null, 2));
console.log(`${osName}: median=${round(median)}ms (±${round(iqr)}ms)`);
