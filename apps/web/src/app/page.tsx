// SPDX-License-Identifier: GPL-3.0-or-later
import Link from 'next/link';
import { CapabilityProbe } from '@/components/capability-probe';
import { FEATURES } from '@/lib/content';

/** The four claims, each stating the mechanism rather than the adjective. */
const CLAIMS = [
  {
    title: 'Cost scales with change',
    body: 'Moving one object in a scene of a million uploads one record. The renderer keeps the scene on the GPU and receives deltas, so a frame costs what changed plus what is visible.',
    accent: 'near',
  },
  {
    title: 'The GPU decides what to draw',
    body: 'Culling and level-of-detail selection run in compute shaders and generate draw commands directly. The CPU submits a camera and a graph, not a loop over every object.',
    accent: 'far',
  },
  {
    title: 'The same project opens everywhere',
    body: 'Desktop, iPad, Android, and the browser run the same core and the same file. No mobile-lite format, no export step, no cloud round trip.',
    accent: 'near',
  },
  {
    title: 'Saving is a transaction',
    body: 'Changes are journalled and written as chunks. Killing the process is a supported way to exit, and recovery is a tested path rather than a hope.',
    accent: 'far',
  },
] as const;

export default function HomePage(): React.JSX.Element {
  const shipped = FEATURES.filter((feature) => feature.state === 'shipped');

  return (
    <>
      <section className="border-b border-rule px-4 pb-16 pt-16 sm:px-8 sm:pb-24 sm:pt-24">
        <div className="grid gap-14 lg:grid-cols-[minmax(0,1.1fr)_minmax(0,1fr)] lg:gap-16">
          <div>
            <p className="eyebrow">Open source · Rust · GPL-3.0-or-later</p>
            <h1 className="display mt-5 text-[clamp(2.75rem,8vw,5.5rem)] text-chalk">
              3D creation
              <br />
              that reports
              <br />
              <span className="text-near">its own numbers</span>
            </h1>
            <p className="mt-7 max-w-xl text-lg leading-relaxed text-mute">
              A modelling, scene-assembly, and look-development environment for desktop, iPad,
              Android, and the browser. Every performance claim it makes has a public scene, a
              stated device, and a command that reproduces it.
            </p>

            <div className="mt-9 flex flex-wrap items-center gap-4">
              <Link
                href="/download"
                className="bg-near px-6 py-3 font-mono text-sm font-medium text-void transition-colors hover:bg-chalk"
              >
                Get Topovium
              </Link>
              <Link
                href="/benchmarks"
                className="border border-rule px-6 py-3 font-mono text-sm text-chalk transition-colors hover:border-near hover:text-near"
              >
                See the measurements
              </Link>
            </div>

            <p className="mt-8 max-w-xl border-l-2 border-stall pl-4 text-sm leading-relaxed text-mute">
              <span className="font-mono text-stall">Pre-alpha.</span> Nothing here is ready for
              real work. {shipped.length} pieces of the core are built and tested; the editor on
              top of them is not. This page will always say so.
            </p>
          </div>

          <div className="lg:pt-16">
            <CapabilityProbe />
            <p className="mt-4 max-w-md text-xs leading-relaxed text-mute">
              Read from your browser through WebGPU, in the same shape{' '}
              <code className="font-mono text-chalk">topovium capability-dump</code> prints. Nothing
              is sent anywhere.
            </p>
          </div>
        </div>
      </section>

      <section className="border-b border-rule px-4 py-16 sm:px-8 sm:py-24">
        <h2 className="eyebrow">Why it can be faster</h2>
        <p className="mt-4 max-w-2xl text-lg text-chalk">
          Not by working harder at the same design. By making four structural choices that are
          difficult to retrofit into an application that did not start with them.
        </p>

        <div className="mt-12 grid gap-px border border-rule bg-rule sm:grid-cols-2">
          {CLAIMS.map((claim) => (
            <article key={claim.title} className="bg-void p-6 sm:p-8">
              <span
                aria-hidden
                className={`block h-px w-10 ${claim.accent === 'near' ? 'bg-near' : 'bg-far'}`}
              />
              <h3 className="display mt-5 text-2xl text-chalk">{claim.title}</h3>
              <p className="mt-4 text-sm leading-relaxed text-mute">{claim.body}</p>
            </article>
          ))}
        </div>
      </section>

      <section className="px-4 py-16 sm:px-8 sm:py-24">
        <div className="grid gap-12 lg:grid-cols-[minmax(0,1fr)_minmax(0,1.2fr)]">
          <div>
            <h2 className="eyebrow">Four platform families</h2>
            <p className="display mt-4 text-4xl text-chalk">One core, four surfaces</p>
            <p className="mt-5 max-w-md text-sm leading-relaxed text-mute">
              The native shells handle lifecycle, files, pen and touch input, and thermal
              callbacks — a few hundred lines each. Everything above them, including all the
              tools and the interface, is shared Rust.
            </p>
          </div>

          <dl className="divide-y divide-rule border-y border-rule">
            {[
              { platform: 'Windows · Linux · macOS', shell: 'Rust', backend: 'DX12 · Vulkan · Metal' },
              { platform: 'iPadOS', shell: 'Swift, thin', backend: 'Metal' },
              { platform: 'Android', shell: 'Kotlin, thin', backend: 'Vulkan' },
              { platform: 'Browser', shell: 'TypeScript, thin', backend: 'WebGPU' },
            ].map((row) => (
              <div
                key={row.platform}
                className="grid grid-cols-1 gap-1 py-4 sm:grid-cols-[minmax(0,1.4fr)_minmax(0,0.8fr)_minmax(0,1.2fr)] sm:gap-4"
              >
                <dt className="text-sm text-chalk">{row.platform}</dt>
                <dd className="font-mono text-xs text-mute">{row.shell}</dd>
                <dd className="font-mono text-xs text-far">{row.backend}</dd>
              </div>
            ))}
          </dl>
        </div>
      </section>
    </>
  );
}
