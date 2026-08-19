// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * Mirrors `CapabilityReport` in `crates/diagnostics`.
 *
 * Kept in sync by hand for now; 0.0.2 generates it from `schemas/`, at which point a
 * drift between Rust and TypeScript becomes a build failure rather than a wrong number
 * on a public page.
 */
export interface CapabilityReport {
  readonly schemaVersion: number;
  readonly backend: Backend;
  readonly adapterName: string;
  readonly driverInfo: string;
  readonly tier: DeviceTier;
  readonly unifiedMemory: boolean;
  readonly supportsTimestampQueries: boolean;
  readonly supportsRayQuery: boolean;
  readonly supportsMultiDrawIndirect: boolean;
  readonly maxTextureDimension2d: number;
}

export type Backend = 'vulkan' | 'metal' | 'dx12' | 'webgpu' | 'cpu';

export type DeviceTier = 'basic' | 'modern' | 'high-end';

/** What each tier means for the person reading it, in their terms. */
export const TIER_MEANING: Record<DeviceTier, string> = {
  basic: 'Full editing, simplified rendering. Heavy renders run on another machine.',
  modern: 'Full PBR viewport, GPU culling, texture streaming, interactive preview.',
  'high-end': 'Everything in Modern, plus ray queries and GPU denoising where stable.',
};

/** Delivery state of a capability. The site never claims more than this. */
export type FeatureState = 'shipped' | 'in-progress' | 'planned';

export interface Feature {
  readonly name: string;
  readonly state: FeatureState;
  readonly milestone: string;
  readonly detail: string;
}

export interface Milestone {
  readonly version: string;
  readonly title: string;
  readonly phase: string;
  /** The condition that closes this milestone. Not a summary -- a test. */
  readonly gate: string;
  readonly state: FeatureState;
}
