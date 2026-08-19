# ADR-0001: Rust core with a wgpu baseline and native fast paths

- **Status:** accepted
- **Date:** 2026-08-20

## Context

Topovium targets Windows, Linux, macOS, iPadOS, Android, and the browser. Those platforms
expose DX12, Vulkan, Metal, and WebGPU. Writing four renderers from the start is not
affordable; writing one that ignores their differences produces a lowest-common-denominator
renderer that is slow everywhere and cannot use hardware ray tracing, mesh shaders, or
sparse residency where they exist.

## Decision

The core is Rust. The rendering baseline is `wgpu`, confined to `backends/wgpu-common`,
behind a `RenderDevice` trait in `render-api` that names no graphics API.

Native fast paths are added later as additional implementations of the same trait, not as
special cases inside the scene code.

## Consequences

**Good.** One renderer runs everywhere on day one. The browser target is nearly free.
Memory safety removes a large class of graphics bugs that reproduce on one vendor's driver
only. The boundary is enforced by a CI check rather than by discipline.

**Bad.** `wgpu` lags native APIs on experimental features; ray tracing and mesh shaders are
reported absent in the baseline and reached through fast paths. There is a thin abstraction
cost. A `wgpu` release can break us.

**Rejected: writing directly against each native API.** Four renderers before there is
anything to render, and no browser target at all.

**Rejected: `wgpu` with no abstraction.** Fastest to start, but it makes `wgpu` types
appear throughout the scene code, and adding a native fast path later would then require
rewriting everything above it. The whole value of the boundary is that it is there before
it is needed.
