# ADR-0002: Persistent GPU scene with delta updates

- **Status:** accepted
- **Date:** 2026-08-20
- **Supersedes:** the immutable per-frame snapshot sketched in briefing v0.1

## Context

The straightforward way to keep a renderer and an editor separate is to build an immutable
snapshot of the scene each frame and hand it over. It is easy to reason about and correct
by construction.

It is also `O(total scene size)` per frame. In a scene of one million objects, moving one
of them copies a million records to draw the result. That single property would cap the
product's ambition permanently, because every other optimisation — GPU culling, streaming,
instancing — is downstream of it and none of them help if the CPU already walked every
object.

## Decision

Scene data lives on the GPU across frames. The CPU sends deltas: which object indices
changed, coalesced into contiguous ranges.

The snapshot remains a *logical* boundary — the renderer never reads editor-mutable state
— but is not a physical copy.

## Consequences

**Good.** Moving one object in a million uploads one record. Two thousand adjacent objects
become one write. Frame cost scales with change and visibility, which is the property the
entire architecture depends on. GPU-driven culling becomes worthwhile, because the culling
result no longer has to round-trip through a CPU that already visited every object.

**Bad.** More complex than a snapshot. Change tracking must be correct or the GPU silently
renders stale data — a class of bug that does not crash and may only appear on one machine.
Compaction and index rebuilds still need explicit handling outside the hot path.

**Mitigation.** `crates/gpu-scene` carries direct tests for the properties that matter,
including one asserting that a single change in a million-object scene produces exactly one
range. If that test ever fails, this decision has stopped paying for itself.

**Rejected: full snapshot per frame.** Correct, simple, and permanently `O(scene)`.

**Rejected: double-buffered full copies.** Removes the stall, keeps the copy cost, and
doubles memory.
