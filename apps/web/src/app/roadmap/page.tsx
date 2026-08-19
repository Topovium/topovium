// SPDX-License-Identifier: GPL-3.0-or-later
import type { Metadata } from 'next';
import { MILESTONES } from '@/lib/content';

export const metadata: Metadata = {
  title: 'Roadmap · Topovium',
  description: 'Milestones and the gate that closes each one.',
};

export default function RoadmapPage(): React.JSX.Element {
  return (
    <section className="px-4 py-16 sm:px-8 sm:py-24">
      <p className="eyebrow">Roadmap</p>
      <h1 className="display mt-4 text-[clamp(2.25rem,6vw,4rem)] text-chalk">
        Every milestone has a gate
      </h1>
      <p className="mt-6 max-w-2xl text-lg leading-relaxed text-mute">
        A milestone closes when its gate passes, not when the work feels finished. The gate is a
        condition someone else can check, which is what stops a version number from becoming an
        opinion.
      </p>

      <ol className="mt-14 border-t border-rule">
        {MILESTONES.map((milestone) => {
          const active = milestone.state === 'in-progress';
          return (
            <li
              key={milestone.version}
              className="grid gap-3 border-b border-rule py-7 sm:grid-cols-[minmax(0,7rem)_minmax(0,16rem)_1fr] sm:gap-8"
            >
              <div>
                <p className={`font-mono text-sm ${active ? 'text-near' : 'text-mute'}`}>
                  {milestone.version}
                </p>
                <p className="mt-1 font-mono text-[0.6875rem] text-mute">{milestone.phase}</p>
              </div>
              <h2 className={`text-base ${active ? 'text-chalk' : 'text-mute'}`}>
                {milestone.title}
                {active ? (
                  <span className="ml-2 font-mono text-[0.6875rem] text-near">in progress</span>
                ) : null}
              </h2>
              <p className="text-sm leading-relaxed text-mute">
                <span className="font-mono text-[0.6875rem] uppercase tracking-widest text-far">
                  Gate ·{' '}
                </span>
                {milestone.gate}
              </p>
            </li>
          );
        })}
      </ol>

      <p className="mt-10 max-w-2xl text-sm text-mute">
        The live issue list is on{' '}
        <a
          className="text-near underline underline-offset-4 hover:text-chalk"
          href="https://github.com/Topovium/topovium/milestones"
        >
          GitHub
        </a>
        . Where this page and the milestones disagree, GitHub is right and this is a bug.
      </p>
    </section>
  );
}
