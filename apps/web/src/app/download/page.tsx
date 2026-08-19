// SPDX-License-Identifier: GPL-3.0-or-later
import type { Metadata } from 'next';
import { DOWNLOADS } from '@/lib/content';

export const metadata: Metadata = {
  title: 'Download · Topovium',
  description: 'Builds for Windows, Linux, macOS, iPadOS, Android, and the browser.',
};

export default function DownloadPage(): React.JSX.Element {
  return (
    <section className="px-4 py-16 sm:px-8 sm:py-24">
      <p className="eyebrow">Download</p>
      <h1 className="display mt-4 text-[clamp(2.25rem,6vw,4rem)] text-chalk">
        Nothing to download yet
      </h1>
      <p className="mt-6 max-w-2xl text-lg leading-relaxed text-mute">
        Topovium is at 0.0.1 and has no editor to ship. Publishing an installer now would
        waste your time and cost us the only thing this project has, which is being straight
        with you about what works.
      </p>
      <p className="mt-4 max-w-2xl text-lg leading-relaxed text-mute">
        Builds appear here when the 0.0.2 gate passes: edit a cube, save it, kill the process,
        and recover the project — on every platform. Until then you can build from source.
      </p>

      <div className="mt-10 flex flex-wrap gap-4">
        <a
          href="https://github.com/Topovium/topovium"
          className="bg-near px-6 py-3 font-mono text-sm font-medium text-void transition-colors hover:bg-chalk"
        >
          Build from source
        </a>
        <a
          href="https://github.com/Topovium/topovium/milestone/2"
          className="border border-rule px-6 py-3 font-mono text-sm text-chalk transition-colors hover:border-near hover:text-near"
        >
          Track the 0.0.2 milestone
        </a>
      </div>

      <h2 className="eyebrow mt-20">Planned targets and requirements</h2>
      <dl className="mt-6 divide-y divide-rule border-y border-rule">
        {DOWNLOADS.map((target) => (
          <div
            key={target.platform}
            className="grid grid-cols-1 items-baseline gap-1 py-4 sm:grid-cols-[minmax(0,10rem)_1fr_auto] sm:gap-6"
          >
            <dt className="text-base text-chalk">{target.platform}</dt>
            <dd className="text-sm text-mute">{target.requirement}</dd>
            <dd className="font-mono text-xs text-stall">
              {target.available ? 'available' : 'not yet built'}
            </dd>
          </div>
        ))}
      </dl>

      <h2 className="eyebrow mt-20">Building it yourself</h2>
      <pre className="mt-6 overflow-x-auto border border-rule bg-slate p-5 font-mono text-xs leading-relaxed text-chalk">
        <code>{`git clone https://github.com/Topovium/topovium
cd topovium

just check        # fmt, clippy, tests, boundaries, dependency audit
just capability   # what your GPU can actually do
just run          # the desktop viewport`}</code>
      </pre>
      <p className="mt-4 max-w-2xl text-sm text-mute">
        Requires the toolchain pinned in <code className="font-mono text-chalk">rust-toolchain.toml</code>,
        which <code className="font-mono text-chalk">rustup</code> installs for you, plus Node 22
        and pnpm for the site.
      </p>
    </section>
  );
}
