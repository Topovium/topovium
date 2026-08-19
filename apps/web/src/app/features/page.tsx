// SPDX-License-Identifier: GPL-3.0-or-later
import type { Metadata } from 'next';
import { FEATURES } from '@/lib/content';
import type { FeatureState } from '@/lib/types';

export const metadata: Metadata = {
  title: 'Features · Topovium',
  description: 'What exists, what is being built, and what is only planned.',
};

const STATE_STYLE: Record<FeatureState, string> = {
  shipped: 'text-near',
  'in-progress': 'text-chalk',
  planned: 'text-mute',
};

const STATE_LABEL: Record<FeatureState, string> = {
  shipped: 'built and tested',
  'in-progress': 'in progress',
  planned: 'planned',
};

const ORDER: readonly FeatureState[] = ['shipped', 'in-progress', 'planned'];

export default function FeaturesPage(): React.JSX.Element {
  return (
    <section className="px-4 py-16 sm:px-8 sm:py-24">
      <p className="eyebrow">Features</p>
      <h1 className="display mt-4 text-[clamp(2.25rem,6vw,4rem)] text-chalk">
        What actually exists
      </h1>
      <p className="mt-6 max-w-2xl text-lg leading-relaxed text-mute">
        Anything absent from this page does not exist. Listing intentions as capabilities is the
        fastest way to lose an audience that has been promised a Blender replacement before.
      </p>

      {ORDER.map((state) => {
        const group = FEATURES.filter((feature) => feature.state === state);
        if (group.length === 0) return null;

        return (
          <div key={state} className="mt-16">
            <h2 className="eyebrow flex items-baseline gap-3">
              <span className={STATE_STYLE[state]}>{STATE_LABEL[state]}</span>
              <span className="text-mute">{group.length}</span>
            </h2>

            <ul className="mt-5 divide-y divide-rule border-y border-rule">
              {group.map((feature) => (
                <li key={feature.name} className="grid gap-2 py-5 sm:grid-cols-[minmax(0,18rem)_1fr] sm:gap-8">
                  <div>
                    <h3 className={`text-base ${STATE_STYLE[state]}`}>{feature.name}</h3>
                    <p className="mt-1 font-mono text-[0.6875rem] text-mute">{feature.milestone}</p>
                  </div>
                  <p className="text-sm leading-relaxed text-mute">{feature.detail}</p>
                </li>
              ))}
            </ul>
          </div>
        );
      })}

      <h2 className="eyebrow mt-20">What Topovium is not</h2>
      <ul className="mt-5 grid gap-3 border-l-2 border-stall pl-5 text-sm leading-relaxed text-mute">
        <li>Not a drop-in Blender replacement. It will not open .blend files or run Blender addons.</li>
        <li>Not a full effects suite. No fluids, destruction, crowds, or compositor in the 0.0.x line.</li>
        <li>Not a character animation package yet.</li>
        <li>Not faster than everything at everything. Where a competitor wins, our benchmarks say so.</li>
        <li>Not cloud-dependent. Every core workflow works offline, with no account, forever.</li>
      </ul>
    </section>
  );
}
