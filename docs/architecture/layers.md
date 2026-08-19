# Layer contract

Dependencies flow downward only. Enforced by `cargo xtask check-boundaries`, which runs
in CI, because an architecture diagram is a wish until something fails the build.

```
apps/desktop  apps/cli  apps/ipad-shell  apps/android-shell  apps/web
      │            │            │                │              │
      └────────────┴────────────┴────────────────┴──────────────┘
                              │
                        crates/viewport
                              │
                      crates/gpu-scene
                              │
                    crates/render-graph
                              │
                     crates/render-api          ← the boundary
                              │
                 backends/wgpu-common           ← the only crate that names wgpu
                              │
                            wgpu

  leaves, depended on by everything, depending on nothing in this workspace:
  crates/foundation   crates/revisions   crates/job-system   crates/platform-api
```

## Forbidden edges

| Edge | Why |
|---|---|
| Anything except `backends/wgpu-common` naming a `wgpu` type | A native Metal or DX12 fast path is planned. When it arrives, no scene, tool, or viewport code should change. |
| `render-api` exposing a backend type in its public API | Same reason, one layer up. The boundary is only a boundary if nothing crosses it. |
| `platform-api` naming UIKit, JNI, `winit`, or the DOM | It describes what a shell must provide without naming a platform. A concrete type here defeats the abstraction and forces every tool to branch on operating system. |
| `foundation` or `revisions` depending on any workspace crate | They are leaves. A cycle through them would make the whole graph untestable in isolation. |
| `ffi` exposing internal scene structures | The ABI must not change when a Rust struct gains a field. |

## Patterns that are rejected regardless of where they appear

| Pattern | Why |
|---|---|
| `Arc<Mutex<T>>` per scene object | A million objects means a million allocations and a million locks. Use handles into columnar storage. |
| A global scene lock | It serialises exactly the work that needed to be parallel. |
| A full-scene copy per frame | Cost must scale with what changed and what is visible, never with total scene size. |
| A CPU loop over all objects as shipped architecture | Culling and LOD selection belong on the GPU. |
| Blocking the UI thread on save, shader compilation, BVH, or import | If it can take 16 ms, it goes to the job system. |

## The three-revision pipeline

Each stage reads data no other stage is writing, so no stage waits on another:

```
authoring   revision N+1     what the user is editing right now
evaluated   revision N       modifier and animation results
gpu         revision N-1     what the GPU is currently drawing
```

Input never waits on the GPU. A job that finishes late carries the revision of its inputs
and is discarded if the scene has moved on, rather than silently reverting the user's most
recent edit.
