// SPDX-License-Identifier: GPL-3.0-or-later
import type { Feature, Milestone } from './types';

/**
 * The roadmap, mirroring the GitHub milestones.
 *
 * Each entry carries the gate that closes it rather than a description, so a milestone
 * cannot be quietly declared done. If a gate here and the GitHub milestone disagree,
 * the GitHub milestone is right and this is a bug.
 */
export const MILESTONES: readonly Milestone[] = [
  {
    version: '0.0.1',
    title: 'Performance Lab',
    phase: 'Phase 0',
    gate: 'The same minimal renderer runs on desktop, iPad, Android, and the web, with timings measured automatically.',
    state: 'in-progress',
  },
  {
    version: '0.0.2',
    title: 'Cross-Device Cube Editor',
    phase: 'Phase 1',
    gate: 'Edit a cube, save, kill the process, and recover the project -- on every platform.',
    state: 'planned',
  },
  {
    version: '0.0.3',
    title: 'Modelling Vertical Slice',
    phase: 'Phase 2',
    gate: 'A user produces a complete simple asset without opening another program.',
    state: 'planned',
  },
  {
    version: '0.0.4',
    title: 'GPU-Driven Viewport',
    phase: 'Phase 3',
    gate: 'A large-instance scene is limited by the GPU, not by per-object CPU work.',
    state: 'planned',
  },
  {
    version: '0.0.5',
    title: 'Materials, Streaming, Mobile',
    phase: 'Phase 4',
    gate: 'Start an asset on a tablet, refine it on desktop, and reopen it on mobile.',
    state: 'planned',
  },
  {
    version: '0.0.6',
    title: 'Interactive Rendering',
    phase: 'Phase 5',
    gate: 'A readable light and material preview appears within a stated time budget.',
    state: 'planned',
  },
];

/**
 * What exists, honestly.
 *
 * Anything not listed here does not exist. A features page that lists intentions as
 * capabilities is the fastest way to lose the audience this project needs.
 */
export const FEATURES: readonly Feature[] = [
  {
    name: 'Generation-checked handles',
    state: 'shipped',
    milestone: '0.0.1',
    detail:
      'A reference to a deleted object reports itself as stale rather than silently reading whatever took its place.',
  },
  {
    name: 'Typed dirty domains',
    state: 'shipped',
    milestone: '0.0.1',
    detail:
      'Changing roughness does not invalidate geometry. Moving a light does not rebuild meshlets.',
  },
  {
    name: 'Coalesced delta uploads',
    state: 'shipped',
    milestone: '0.0.1',
    detail:
      'Moving one object in a scene of a million uploads one record. Two thousand adjacent objects become one write.',
  },
  {
    name: 'Adaptive frame budget',
    state: 'shipped',
    milestone: '0.0.1',
    detail:
      'Quality drops in small ordered steps when frames run long, and recovers slowly so it does not visibly oscillate.',
  },
  {
    name: 'Priority job scheduling',
    state: 'shipped',
    milestone: '0.0.1',
    detail:
      'Work the user is waiting on outranks everything. Background work throttles under thermal pressure; interactive work never does.',
  },
  {
    name: 'Transient memory aliasing',
    state: 'shipped',
    milestone: '0.0.1',
    detail:
      'Render targets that are never live at the same time share memory. On a tablet this is the difference between fitting and being killed.',
  },
  {
    name: 'Desktop viewport',
    state: 'in-progress',
    milestone: '0.0.1',
    detail: 'A window, a GPU device, and a measured frame loop on Windows, Linux, and macOS.',
  },
  {
    name: 'iPad and Android shells',
    state: 'in-progress',
    milestone: '0.0.1',
    detail: 'Thin native shells over the shared Rust core. Lifecycle, files, pen, and thermal callbacks.',
  },
  {
    name: 'Browser viewport',
    state: 'in-progress',
    milestone: '0.0.1',
    detail: 'The same core compiled to WebAssembly against WebGPU. No competitor ships one.',
  },
  {
    name: 'Transactional save and recovery',
    state: 'planned',
    milestone: '0.0.2',
    detail:
      'Journalled changes and chunked writes. Killing the process is a supported way to exit.',
  },
  {
    name: 'Polygon modelling tools',
    state: 'planned',
    milestone: '0.0.3',
    detail: 'Selection, move, rotate, scale, extrude, inset, bevel, with local mesh updates.',
  },
  {
    name: 'Interactive path tracing',
    state: 'planned',
    milestone: '0.0.6',
    detail: 'A readable preview of light and material inside a fixed time budget.',
  },
];

export interface DownloadTarget {
  readonly platform: string;
  readonly requirement: string;
  readonly available: boolean;
}

export const DOWNLOADS: readonly DownloadTarget[] = [
  { platform: 'Windows', requirement: 'Windows 10 or later, DX12 or Vulkan', available: false },
  { platform: 'Linux', requirement: 'Vulkan 1.2 drivers', available: false },
  { platform: 'macOS', requirement: 'macOS 13 or later, Apple silicon or Intel', available: false },
  { platform: 'iPadOS', requirement: 'iPadOS 17 or later', available: false },
  { platform: 'Android', requirement: 'Android 10 or later with Vulkan', available: false },
  { platform: 'Browser', requirement: 'Any browser exposing WebGPU', available: false },
];
