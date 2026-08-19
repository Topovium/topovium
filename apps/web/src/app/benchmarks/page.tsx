// SPDX-License-Identifier: GPL-3.0-or-later
import type { Metadata } from 'next';

export const metadata: Metadata = {
  title: 'Benchmarks · Topovium',
  description:
    'Published measurements: scene, device, driver, resolution, p50/p95/p99, and the command that reproduces each one.',
};

/** The rules, reproduced from docs/performance/benchmark-contract.md. */
const RULES = [
  {
    rule: 'Percentiles, never averages',
    why: 'A mean frame time of 8 ms hides the 90 ms hitch that is the only thing the user noticed. Every result reports p50, p95, and p99.',
  },
  {
    rule: 'One command reproduces it',
    why: 'Each published number carries the exact command that produced it. A number nobody can reproduce is not evidence.',
  },
  {
    rule: 'Identical hardware and settings',
    why: 'Scene, device, driver version, resolution, and quality level are recorded. A faster run at lower quality is not a faster run, and the comparison tool refuses it.',
  },
  {
    rule: 'Public scenes',
    why: 'Every scene is generated from a seed rather than shipped as a binary, so anyone can regenerate exactly what we measured.',
  },
  {
    rule: 'Regressions are published',
    why: 'Including the cases where another application is faster. A page that only ever shows us winning is marketing, and nobody believes marketing.',
  },
  {
    rule: 'No single best run',
    why: 'Results come from full runs, not the best of several. Cherry-picking is the most common way a benchmark lies.',
  },
] as const;

const SCENES = [
  {
    id: 'startup-empty',
    what: 'Cold start to an editable viewport with an empty project.',
    measures: 'Startup time, first frame, peak memory.',
  },
  {
    id: 'instances-100k',
    what: '100,000 instances of one mesh, one material.',
    measures: 'Whether per-object CPU work has crept back in.',
  },
  {
    id: 'transform-single-in-million',
    what: 'One object moved per frame in a scene of one million.',
    measures: 'The headline claim: this must cost one record, not a million.',
  },
] as const;

export default function BenchmarksPage(): React.JSX.Element {
  return (
    <section className="px-4 py-16 sm:px-8 sm:py-24">
      <p className="eyebrow">Benchmarks</p>
      <h1 className="display mt-4 text-[clamp(2.25rem,6vw,4rem)] text-chalk">
        No results published yet
      </h1>
      <p className="mt-6 max-w-2xl text-lg leading-relaxed text-mute">
        The harness, the result schema, and the regression gate are built. The renderer they
        are meant to measure is not, so there is nothing honest to put here. This page fills in
        automatically from nightly runs once 0.0.1 closes.
      </p>
      <p className="mt-4 max-w-2xl text-lg leading-relaxed text-mute">
        Publishing a number before then would break the first rule below, on the page whose
        entire purpose is that rule.
      </p>

      <h2 className="eyebrow mt-20">The contract</h2>
      <p className="mt-3 max-w-2xl text-sm text-mute">
        These apply to our own results, not only to comparisons. A merge is blocked if p95 or
        p99 regresses beyond tolerance, if memory grows past its threshold, or if a new stall
        appears.
      </p>
      <ol className="mt-8 divide-y divide-rule border-y border-rule">
        {RULES.map((entry) => (
          <li key={entry.rule} className="grid gap-2 py-5 sm:grid-cols-[minmax(0,16rem)_1fr] sm:gap-8">
            <h3 className="text-base text-chalk">{entry.rule}</h3>
            <p className="text-sm leading-relaxed text-mute">{entry.why}</p>
          </li>
        ))}
      </ol>

      <h2 className="eyebrow mt-20">Baseline scenes</h2>
      <ul className="mt-6 grid gap-px border border-rule bg-rule sm:grid-cols-3">
        {SCENES.map((scene) => (
          <li key={scene.id} className="bg-void p-6">
            <p className="font-mono text-xs text-near">{scene.id}</p>
            <p className="mt-3 text-sm text-chalk">{scene.what}</p>
            <p className="mt-3 text-xs leading-relaxed text-mute">{scene.measures}</p>
          </li>
        ))}
      </ul>

      <h2 className="eyebrow mt-20">Run them yourself</h2>
      <pre className="mt-6 overflow-x-auto border border-rule bg-slate p-5 font-mono text-xs leading-relaxed text-chalk">
        <code>{`just scenes                          # list every scene this build can run
just bench instances-100k            # run one and emit benchmark-result.json
just capability                      # the machine the numbers came from`}</code>
      </pre>
    </section>
  );
}
