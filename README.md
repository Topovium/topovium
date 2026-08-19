<div align="center">

# Topovium

**A 3D creation environment for desktop, tablet, and the browser.**

Written in Rust. One project file, five platforms, no account required.

[![CI](https://github.com/Topovium/topovium/actions/workflows/ci.yml/badge.svg)](https://github.com/Topovium/topovium/actions/workflows/ci.yml)
[![License: GPL v3](https://img.shields.io/badge/license-GPL--3.0--or--later-blue.svg)](./LICENSE)

</div>

---

> **Status: pre-alpha, 0.0.1.**
> Nothing here is usable for real work yet. This section will always say honestly what
> does and does not run. See [what works today](#what-works-today).

## What Topovium is

A modelling, scene-assembly, and look-development environment built around four ideas
that are hard to add to an existing application and therefore worth building from
scratch:

**Cost scales with change, not with scene size.** Moving one object in a scene of a
million must not touch the other 999,999. The renderer keeps a persistent scene on the
GPU and receives deltas, so a frame costs what changed plus what is visible.

**The GPU decides what to draw.** Frustum culling, occlusion culling, and LOD selection
run in compute shaders and generate draw commands directly. The CPU submits a camera
and a graph, not a loop over every object.

**The same project opens everywhere.** Desktop, iPad, Android, and the browser run the
same Rust core and the same file. No mobile-lite format, no export step, no cloud
round-trip. Weaker devices render more simply; none of them lose your data.

**Saving is a transaction.** Changes are journalled and written as chunks. Killing the
process is a supported way to exit, and recovery is a tested path rather than a hope.

## What Topovium is not

Not yet, and some never:

- Not a drop-in Blender replacement. It will not open `.blend` files or run Blender addons.
- Not a full VFX suite. No fluids, destruction, crowds, or compositor in the 0.0.x line.
- Not a character animation package yet.
- Not faster than everything at everything. Where a competitor wins, [our own benchmarks
  say so](./docs/performance/benchmark-contract.md).
- Not cloud-dependent. Every core workflow works offline, forever, with no account.

## Platforms

| Platform | Surface | Shell | Backend |
|---|---|---|---|
| Windows / Linux / macOS | `winit` window | Rust | DX12 · Vulkan · Metal |
| iPadOS | `CAMetalLayer` | Swift, thin | Metal |
| Android | `SurfaceView` | Kotlin, thin | Vulkan |
| Web | `<canvas>` | TypeScript, thin | WebGPU |

The native shells handle lifecycle, files, pen and touch input, IME, and thermal
callbacks — a few hundred lines each. Everything above that, including all UI and
tools, is shared Rust.

## What works today

**0.0.1 is in progress.** Honest state:

- [x] Cargo workspace, ten core crates, tests, and lint gates
- [x] Generation-checked handle arena, revisions, typed dirty domains, change sets
- [x] Priority job pool with cancellation and thermal throttling
- [x] Frame log with p50/p95/p99 and stall detection
- [x] Capability report derived from measured features
- [ ] Desktop window rendering a triangle through `wgpu`
- [ ] iPad and Android shells
- [ ] WebGPU build running in a browser
- [ ] Benchmark harness publishing results
- [ ] Persistent GPU scene with delta updates

Track it on the [0.0.1 milestone](https://github.com/Topovium/topovium/milestone/1).

## Building

Requires the toolchain in [`rust-toolchain.toml`](./rust-toolchain.toml) (installed
automatically by `rustup`), Node 22+, and `pnpm`.

```sh
just check          # fmt, clippy, tests, boundary and dependency checks
just run            # desktop editor
just capability     # dump this machine's GPU capability report as JSON
just web            # website + WASM viewport at http://localhost:3000
```

`just --list` shows everything. There is no step in this project that is not a `just`
recipe — if you find one, that is a bug worth filing.

## Repository layout

```
apps/        desktop · cli · ipad-shell · android-shell · web (topovium.org)
crates/      the Rust core — foundation, revisions, gpu-scene, viewport, ...
backends/    the only place wgpu types are allowed to appear
benchmarks/  the harness and the scenes every performance claim is measured on
docs/        vision · architecture · adr · rfc · performance
schemas/     JSON Schema for benchmark results and the capability report
tools/       xtask: boundary checks, codegen, cross-platform builds
```

## Performance claims

Every one of them has a public scene, a stated device, a driver version, p50/p95/p99,
and a command that reproduces it. Results are published — including regressions, and
including the cases where another application is faster.

The rules are written down in
[`docs/performance/benchmark-contract.md`](./docs/performance/benchmark-contract.md)
and they apply to us, not just to comparisons.

## Contributing

Read **[AGENTS.md](./AGENTS.md)** first. It is short, and it is binding on humans and
AI assistants alike. The rules that surprise people most often:

- AI assistants **never commit**. They propose diffs; a human reads and commits them.
- This is a public repository. No key, token, or `.env` file, ever, in any branch.
- No `any` in TypeScript. No `unwrap` outside tests. No `unsafe` without an ADR.
- A performance claim without a benchmark does not merge.

Then see [CONTRIBUTING.md](./CONTRIBUTING.md) for the workflow, and
[docs/architecture/](./docs/architecture/) for how the layers fit together.

## Licence

[GPL-3.0-or-later](./LICENSE).

Chosen so that improvements to Topovium stay available to the people who use it. If
you ship a modified version, you share your changes.
