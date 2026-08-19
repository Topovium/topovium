// SPDX-License-Identifier: GPL-3.0-or-later
'use client';

import { useEffect, useState } from 'react';
import type { DeviceTier } from '@/lib/types';
import { TIER_MEANING } from '@/lib/types';

interface ProbeRow {
  readonly label: string;
  readonly value: string;
  readonly tone: 'normal' | 'near' | 'stall';
}

type ProbeState =
  | { readonly kind: 'probing' }
  | { readonly kind: 'unsupported'; readonly reason: string }
  | { readonly kind: 'ready'; readonly rows: readonly ProbeRow[]; readonly tier: DeviceTier };

/**
 * Classifies the adapter using the same rules as `CapabilityReport::classify` in
 * `crates/diagnostics`. Kept deliberately identical, including the reason Metal-class
 * devices are not demoted for lacking multi-draw-indirect.
 */
function classify(
  supportsRayQuery: boolean,
  supportsMultiDrawIndirect: boolean,
  supportsTimestampQueries: boolean,
  maxTextureDimension2d: number,
): DeviceTier {
  const desktopClass = maxTextureDimension2d >= 16384;
  const modern =
    (desktopClass && supportsTimestampQueries) ||
    (supportsMultiDrawIndirect && maxTextureDimension2d >= 8192);

  if (supportsRayQuery && desktopClass) return 'high-end';
  if (modern) return 'modern';
  return 'basic';
}

function formatBytes(bytes: number): string {
  const gigabytes = bytes / 1024 ** 3;
  if (gigabytes >= 1) return `${gigabytes.toFixed(1)} GB`;
  return `${Math.round(bytes / 1024 ** 2)} MB`;
}

/**
 * The signature element: the visitor's own machine, read through WebGPU and printed in
 * the same shape as `topovium capability-dump`.
 *
 * Nothing here is invented. If the browser exposes no adapter, it says so plainly
 * rather than showing a plausible-looking placeholder -- a project whose whole argument
 * is measurement cannot open with fabricated measurements.
 */
export function CapabilityProbe(): React.JSX.Element {
  const [state, setState] = useState<ProbeState>({ kind: 'probing' });

  useEffect(() => {
    let cancelled = false;

    async function probe(): Promise<void> {
      if (!('gpu' in navigator)) {
        if (!cancelled) {
          setState({
            kind: 'unsupported',
            reason: 'This browser does not expose WebGPU.',
          });
        }
        return;
      }

      try {
        const adapter = await navigator.gpu.requestAdapter({
          powerPreference: 'high-performance',
        });

        if (cancelled) return;
        if (adapter === null) {
          setState({
            kind: 'unsupported',
            reason: 'WebGPU is present but no adapter was offered.',
          });
          return;
        }

        const features = adapter.features;
        const limits = adapter.limits;
        const timestamps = features.has('timestamp-query');
        const maxTexture = limits.maxTextureDimension2D;
        const tier = classify(false, false, timestamps, maxTexture);

        const info: GPUAdapterInfo = adapter.info;
        const adapterName = info.description || info.device || info.vendor || 'undisclosed';

        setState({
          kind: 'ready',
          tier,
          rows: [
            { label: 'backend', value: 'webgpu', tone: 'normal' },
            { label: 'adapter', value: adapterName, tone: 'normal' },
            { label: 'vendor', value: info.vendor || 'undisclosed', tone: 'normal' },
            { label: 'tier', value: tier, tone: 'near' },
            {
              label: 'timestamp queries',
              value: timestamps ? 'yes' : 'no',
              tone: timestamps ? 'near' : 'stall',
            },
            { label: 'max texture 2d', value: `${maxTexture}`, tone: 'normal' },
            {
              label: 'max buffer size',
              value: formatBytes(limits.maxBufferSize),
              tone: 'normal',
            },
            {
              label: 'max workgroup invocations',
              value: `${limits.maxComputeInvocationsPerWorkgroup}`,
              tone: 'normal',
            },
          ],
        });
      } catch (error: unknown) {
        if (cancelled) return;
        const reason = error instanceof Error ? error.message : 'The adapter request failed.';
        setState({ kind: 'unsupported', reason });
      }
    }

    void probe();
    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <div className="border border-rule bg-slate">
      <div className="flex items-baseline justify-between border-b border-rule px-4 py-3 sm:px-5">
        <h2 className="eyebrow">Your machine, as Topovium sees it</h2>
        <span className="font-mono text-[0.6875rem] text-mute">
          {state.kind === 'probing' ? 'reading…' : 'live'}
        </span>
      </div>

      {state.kind === 'probing' ? (
        <p className="px-4 py-6 font-mono text-sm text-mute sm:px-5">
          Requesting a graphics adapter…
        </p>
      ) : null}

      {state.kind === 'unsupported' ? (
        <div className="px-4 py-6 sm:px-5">
          <p className="font-mono text-sm text-stall">{state.reason}</p>
          <p className="mt-3 max-w-prose text-sm text-mute">
            That only affects the browser build. The desktop, iPad, and Android builds talk to
            Vulkan, Metal, and DX12 directly and do not need WebGPU.
          </p>
        </div>
      ) : null}

      {state.kind === 'ready' ? (
        <>
          <dl className="divide-y divide-rule">
            {state.rows.map((row, index) => (
              <div
                key={row.label}
                className="row-in grid grid-cols-[minmax(0,11rem)_1fr] gap-4 px-4 py-2.5 sm:px-5"
                style={{ animationDelay: `${index * 55}ms` }}
              >
                <dt className="font-mono text-xs text-mute">{row.label}</dt>
                <dd
                  className={`truncate font-mono text-xs ${
                    row.tone === 'near'
                      ? 'text-near'
                      : row.tone === 'stall'
                        ? 'text-stall'
                        : 'text-chalk'
                  }`}
                >
                  {row.value}
                </dd>
              </div>
            ))}
          </dl>
          <p className="border-t border-rule px-4 py-3 text-xs leading-relaxed text-mute sm:px-5">
            {TIER_MEANING[state.tier]}
          </p>
        </>
      ) : null}
    </div>
  );
}
