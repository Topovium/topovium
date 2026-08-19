# Topovium Monorepo — Foundation Design

**Date:** 2026-08-20
**Status:** Approved for implementation
**Scope:** Repository foundation, governance rules, and the 0.0.1–0.0.6 milestone/issue structure
**Source:** Briefing strategiczno-technologiczny v0.2

---

## 1. Purpose

Stand up the Topovium repository so that work can begin: a compiling Rust workspace,
a Next.js website, two thin native mobile shells, a web (WASM) target, enforced
engineering rules, and a public roadmap expressed as GitHub milestones and issues.

This document covers the **foundation only**. It does not design the renderer, the
mesh representation, or the project format — those get RFCs (§10).

## 2. Product framing

Topovium is a cross-platform 3D DCC environment written in Rust, targeting desktop,
iPad, Android, and the web from one codebase.

The long-range ambition is a full alternative to Blender, Maya, and Houdini. The
sequencing to get there is deliberate: win decisively and measurably in modeling,
large-scene assembly, look-dev, and cross-device work first, then expand from a
position of proven performance. Every "we are better at X" statement in this project
is backed by a reproducible benchmark or it is not published (§7).

### 2.1 Platform families

Five surface types, one Rust core:

| Family | Surface | Shell | Baseline backend |
|---|---|---|---|
| Desktop | winit window | Rust | wgpu → DX12 / Vulkan / Metal |
| iPadOS | `CAMetalLayer` | Swift, thin | wgpu → Metal |
| Android | `SurfaceView` | Kotlin, thin | wgpu → Vulkan |
| Web | `<canvas>` | TypeScript, thin | wgpu → WebGPU |
| Headless | none | Rust | wgpu → any, or CPU |

"Thin shell" means lifecycle, file access, input devices, IME, and thermal callbacks
only. All UI, tools, scene data, and rendering are shared Rust. Shells are expected
to stay under roughly 500 lines each; growth beyond that is a design smell to be
raised as an issue.

Web is a first-class target from 0.0.1. No competitor ships a browser DCC, so it is
both the cheapest place to be flatly better and a natural funnel from the website.

## 3. Repository layout

```
topovium/
├── AGENTS.md                  golden rules (CLAUDE.md is a pointer to it)
├── README.md LICENSE SECURITY.md CONTRIBUTING.md CODE_OF_CONDUCT.md
├── Cargo.toml                 workspace with explicit members
├── rust-toolchain.toml        pinned toolchain + components + targets
├── deny.toml                  cargo-deny: licenses, bans, advisories
├── justfile                   single entry point for every task
├── package.json               pnpm workspace root
├── pnpm-workspace.yaml
├── .gitignore .gitattributes .editorconfig
├── .github/
│   ├── workflows/             ci · web · mobile · secrets · bench
│   ├── ISSUE_TEMPLATE/        bug · perf-regression · rfc · task
│   ├── PULL_REQUEST_TEMPLATE.md
│   ├── CODEOWNERS
│   └── dependabot.yml
├── apps/
│   ├── desktop/               winit + wgpu, HUD, capability detect
│   ├── cli/                   capability-dump · bench · trace
│   ├── ipad-shell/            Swift, thin
│   ├── android-shell/         Kotlin, thin
│   └── web/                   Next.js 15 — topovium.org + WASM viewport
├── crates/
│   ├── foundation/            math, units, StableHandle, ids, errors
│   ├── revisions/             Revision, Generation, ChangeSet, DirtyDomain
│   ├── diagnostics/           counters, spans, GPU timestamps, capability report
│   ├── job-system/            P0–P4 priorities, cancellation, revision tagging
│   ├── platform-api/          Surface/Lifecycle/Input traits — no backend types
│   ├── render-api/            RenderDevice abstraction + capability query
│   ├── render-graph/          pass graph, transient aliasing
│   ├── gpu-scene/             persistent GPU scene + delta protocol
│   ├── viewport/              frame loop, camera, frame budget manager
│   └── ffi/                   C ABI consumed by Swift and Kotlin shells
├── backends/wgpu-common/      the one place wgpu types are allowed
├── benchmarks/harness/        runs scenes, emits benchmark-result.json
├── tools/xtask/               boundary check, codegen, web/mobile builds
├── schemas/                   JSON Schema: bench results, capability report
├── tests/                     golden-images · recovery · serialization
└── docs/
    ├── vision/                long-range ambition and non-goals
    ├── architecture/          layer contracts, dependency rules
    ├── adr/                   accepted decisions, numbered
    ├── rfc/                   proposals under discussion
    └── performance/           benchmark contract, published results
```

Ten Rust crates, not the thirty in briefing §25. Crates are extracted when a real
responsibility and a contract test exist. The remaining twenty appear as issues in
later milestones rather than as empty directories.

## 4. Layer contract

Dependencies flow downward only. Enforced by `cargo-deny` bans plus
`xtask check-boundaries`, which fails CI on violation.

```
apps/*  →  viewport  →  gpu-scene  →  render-graph  →  render-api  →  backends/wgpu-common
                    ↘  diagnostics ↙
   crates/foundation, revisions, job-system, platform-api are leaves
```

Forbidden edges, verbatim from briefing §25.1 plus the web/FFI additions:

- any crate except `backends/wgpu-common` depending on `wgpu`
- `render-api` naming a concrete backend type in its public API
- `platform-api` depending on Swift, Kotlin, JNI, or DOM concepts
- `foundation` / `revisions` depending on anything above them
- `ffi` exposing internal scene structures rather than handles
- `apps/web` TypeScript reaching into Rust internals except through generated
  `wasm-bindgen` bindings

## 5. `apps/web` — topovium.org

Next.js 15 App Router. TypeScript with `strict`, `noUncheckedIndexedAccess`, and
`exactOptionalPropertyTypes`. Tailwind v4. MDX for documentation. Statically
exported where possible so it hosts on a free tier.

| Route | Purpose |
|---|---|
| `/` | What Topovium is; live WebGPU viewport demo running the real Rust core |
| `/download` | Platform auto-detect, per-OS builds, SHA-256 checksums, release notes, GPU tier requirements |
| `/features` | Honest capability matrix, including what is not implemented yet |
| `/benchmarks` | Published results from `bench.yml`: scene, device, driver, resolution, p50/p95/p99, memory, one-command repro |
| `/docs/[...slug]` | MDX rendered from `docs/` — single source of truth, not a copy |
| `/roadmap` | Generated from the GitHub milestones defined in §8 |
| `/app` | Browser editor. A stub in 0.0.1; grows each milestone |

The benchmarks page is the mechanism that makes performance claims credible. It
publishes regressions and cases where a competitor wins, per briefing §3.3.

## 6. AGENTS.md — golden rules

Four groups. Machine-enforced wherever a check is possible; the rest are review
criteria in the PR template.

### 6.1 Agent conduct

- Agents never run `git commit`, `git push`, `git tag`, `gh pr merge`, or force-push.
- Agents never write to `main`. Branch protection enforces this independently.
- Agents propose diffs; a human reads and commits them.
- Code is not accepted because it compiles. It is accepted because it is understood.
- Agents do not decide: memory model, GPU synchronisation, project format, `unsafe`
  usage, procedural topology semantics, or recovery policy. Those need a human ADR.
- Generated code that is not reviewed line by line does not merge.

### 6.2 Secrets — this is a public repository

- No API keys, tokens, passwords, certificates, `.env` files, keystores, provisioning
  profiles, or service-account JSON. Ever, in any branch, including history.
- Signing material lives only in GitHub Actions secrets.
- `gitleaks` runs on every push and blocks the merge.
- `.gitignore` covers `.env*`, `*.p12`, `*.mobileprovision`, `*.keystore`, `*.jks`.
- A leaked credential is rotated first, scrubbed second. Never the reverse.

### 6.3 TypeScript

- `any` is banned — `@typescript-eslint/no-explicit-any: error`.
- `@ts-ignore` and `@ts-expect-error` are banned outside test fixtures.
- `as` casts require a comment stating why the compiler cannot know.
- `unknown` must be narrowed before crossing a module boundary.
- No non-null assertions (`!`) on values that can genuinely be null.
- No default exports except where a framework requires them.
- `noUncheckedIndexedAccess` and `exactOptionalPropertyTypes` stay on.

### 6.4 Rust and architecture

- `#![forbid(unsafe_code)]` by default; `unsafe` requires an accepted ADR and a
  safety comment per block.
- `unwrap` and `expect` are denied outside `#[cfg(test)]`.
- `clippy -D warnings` on the whole workspace.
- No `Arc<Mutex<_>>` per scene object; no global scene lock.
- No full-scene copy per frame — deltas only.
- The UI thread never blocks on save, shader compilation, BVH build, or import.
- No CPU loop over all objects as a shipped architecture.
- Backend types never escape `render-api`.
- A performance claim without a benchmark scene does not merge.
- Public data formats are versioned with a migration test from every prior version.

## 7. Benchmark contract

`benchmarks/harness` emits JSON validated against `schemas/benchmark-result.schema.json`,
containing at minimum: scene id, device, OS, driver version, backend, resolution,
quality tier, p50/p95/p99 frame time, peak RAM/VRAM/shared memory, and the exact
command to reproduce.

`bench.yml` runs nightly and uploads results; `/benchmarks` renders them. A merge is
blocked on a significant p95/p99 regression, a memory increase past threshold, a new
stall, or a startup-time regression, per briefing §23.3.

Comparison rules (briefing §24.3) are reproduced verbatim in
`docs/performance/benchmark-contract.md`: public scenes, identical hardware, stated
resolution and quality, automated runs, percentiles not averages, and no cherry-picked
single runs.

## 8. Milestones and issues

Six milestones, 0.0.1 through 0.0.6, mapped to briefing phases 0–5. Roughly ten issues
each, about sixty total. Each milestone's description carries its phase gate, so it
cannot be closed on impression.

| Milestone | Phase | Gate |
|---|---|---|
| 0.0.1 Performance Lab | 0 | The same minimal renderer runs on desktop, iPad, Android, and web, with timings measured automatically |
| 0.0.2 Cross-Device Cube Editor | 1 | Edit a cube, save, kill the process, recover the project — on every platform |
| 0.0.3 Modeling Vertical Slice | 2 | A user produces a complete simple asset without another program |
| 0.0.4 GPU-Driven Viewport | 3 | A large-instance scene is GPU-bound, not per-object CPU-bound |
| 0.0.5 Materials, Streaming, Mobile | 4 | Start an asset on tablet, refine on desktop, reopen on mobile |
| 0.0.6 Interactive Rendering | 5 | Readable light and material preview within a stated time budget |

Every issue carries: a one-line outcome, acceptance criteria, the numeric target where
the briefing states one, area and platform labels, and a link to the governing RFC or
ADR when one exists.

Label taxonomy:

- `area:` core · gpu · geometry · materials · ui · platform · web · build · docs
- `platform:` desktop · ipad · android · web · all
- `type:` feat · bug · perf-gate · rfc · chore · test
- `risk:` irreversible — used for decisions that are expensive to change

## 9. CI

| Workflow | Runner | Checks |
|---|---|---|
| `ci.yml` | ubuntu | `fmt --check`, `clippy -D warnings`, `cargo test`, `xtask check-boundaries`, `cargo deny check` |
| `web.yml` | ubuntu | `tsc --noEmit`, eslint, `next build`, WASM build of `viewport` |
| `mobile.yml` | ubuntu + macos | Android NDK cross-build to `aarch64-linux-android`; iOS build to `aarch64-apple-ios` |
| `secrets.yml` | ubuntu | `gitleaks` on push and PR |
| `bench.yml` | nightly | harness run, JSON artifact upload, regression gate |

Branch protection on `main`: required status checks, no force-push, no direct pushes,
linear history.

## 10. Follow-up RFCs

Created as issues in 0.0.1, written as work reaches them. Briefing §37 ordering, with
RFC-0004 first because the viewport, streaming, selection, and renderer all depend on it.

RFC-0001 Product Wedge and Non-Goals · RFC-0002 Platform Capability Contract ·
RFC-0003 Revisions, Handles and ChangeSets · **RFC-0004 Persistent GPU Scene** ·
RFC-0005 Frame Budget and Mobile Thermal Policy · RFC-0006 Editable Mesh, Lineage and
Remapping · RFC-0007 Render API and Native Fast Paths · RFC-0008 Transactional Project
Format · RFC-0009 Benchmark Harness · RFC-0010 Tool Runtime and Cross-Device Input

## 11. Licensing

**GPL-3.0-or-later.** Chosen so that no funded party can take a modified Topovium
closed, and so GPL-licensed prior art can be studied and reused. `LICENSE` holds the
full text; every Rust source file carries an SPDX identifier; `deny.toml` rejects
incoming dependencies with incompatible licenses.

## 12. Non-goals for this foundation work

- No mesh editing, modifiers, materials, or path tracer — those are 0.0.3 onward.
- No plugin ABI. Briefing §31: the scene model stabilises first.
- No account system, telemetry-by-default, or cloud requirement.
- No native DX12/Vulkan/Metal fast paths yet — wgpu baseline first, with the
  abstraction boundary in place so they can be added without touching scene code.
- No marketplace.

## 13. Definition of done for the foundation

- `just check` passes from a clean clone on macOS and Linux.
- `cargo build` succeeds for desktop, and cross-compiles for Android and iOS targets.
- `pnpm build` produces the site; the WASM viewport loads in a WebGPU browser.
- `gitleaks` reports clean.
- `AGENTS.md` exists and every rule in it has either an automated check or an entry
  in the PR template.
- Six milestones exist on GitHub with roughly ten issues each, all labelled and
  carrying acceptance criteria.
- `README.md` states honestly what works today.
