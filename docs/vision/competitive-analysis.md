# Where Topovium can beat Blender, Maya, and Houdini

**Status:** living document
**Last reviewed:** 2026-08-20

The long-range goal is a full alternative to all three. This document is the honest
version of how that could happen: where the incumbents' architecture imposes costs we do
not have to pay, where they are simply better, and which milestone turns each opportunity
into something measurable.

Blender is GPL-2.0-or-later, so Topovium's GPL-3.0-or-later licence permits studying and
even reusing its code. We should study the *decisions* and reuse almost none of the code:
inheriting the architecture is the exact thing we are trying to avoid.

## Structural advantages

Each row is a cost the incumbent pays because of a decision made years ago, and which is
now expensive for them to reverse. These are the only advantages worth planning around —
being better by working harder is not a strategy.

| Their design | The cost it imposes | What Topovium does | Milestone |
|---|---|---|---|
| **Memfile undo** — Blender's global undo snapshots the whole `Main` database into memory | Undo on a large scene is slow and allocates heavily; a long-standing complaint on heavy files | Journalled transactions plus changed chunks. Undo cost scales with the edit, not the file | 0.0.2 |
| **Whole-file save** — every `.blend` write rewrites the entire file | Saving a multi-gigabyte scene after moving one object writes the whole thing | Content-addressed chunks. Moving one object writes one chunk | 0.0.2 |
| **Autosave-only recovery** — a periodic temporary copy | A crash loses up to the autosave interval | Append-only journal with checkpoints. Recovery is a tested path with crash-injection tests | 0.0.2 |
| **BMesh in edit mode** — pointer-based, allocation-heavy, converted BMesh → Mesh → GPU batch on every change | Edit mode degrades badly on dense meshes; the *conversion*, not the edit, dominates | Columnar editable mesh with local dirty ranges. Edit 20 vertices, upload 20 vertices | 0.0.3 |
| **Copy-on-write dependency graph** — evaluation copies whole datablocks | Large scenes pay per-object copies on every evaluation | Typed dirty domains. Moving a light touches nothing geometric; changing roughness invalidates no geometry | shipped |
| **Per-object CPU draw loop** in the draw manager | CPU-bound long before the GPU is | GPU culling and LOD generating indirect draw commands. The CPU submits a camera, not a loop | 0.0.4 |
| **Full-resolution texture loading** at open | Opening a scene with heavy textures stalls and spikes memory | Residency manager with a mip tail always resident and predictive streaming | 0.0.5 |
| **No mobile, no web** — none of the three has either | An artist away from their workstation cannot work | Four platform families from one core and one file | 0.0.1 onward |

The last row deserves emphasis: it is the only place where we can be better on day one
rather than after years of optimisation. Nobody ships a browser DCC. That is a category
we can simply own.

## Where they are better, and will remain so for some time

Writing this down matters as much as the section above. A comparison document that only
lists our advantages is marketing, and the audience for this project can tell.

- **Cycles** is a mature, well-validated production path tracer with years of
  optimisation across CUDA, OptiX, HIP, oneAPI, and Metal. We will not match its feature
  coverage or its quality-per-sample soon.
- **Sculpting.** Multiresolution and dynamic topology sculpting are deep, hard systems.
  Not in the 0.0.x line at all.
- **Geometry nodes** have had years of iteration and an enormous body of user knowledge.
- **The addon ecosystem** is Blender's real moat. Thousands of tools, many essential to
  particular workflows.
- **Houdini's solvers.** Fluids, destruction, and crowds represent decades of specialised
  work. We are not attempting them.
- **Maya's animation and rigging.** The industry standard for character work, with
  pipelines built around it.

## The honest framing

Topovium's plausible path is not "better at everything by 1.0". It is:

1. Be decisively better at modelling, large-scene assembly, look-development, and
   cross-device work — measurably, with published numbers.
2. Be the only one of the four that runs on a tablet and in a browser.
3. Be the one that does not lose your work.
4. Expand from that position, funded by the users those three earn.

Each of those is falsifiable, which is what makes it a plan rather than a wish.

## How each claim gets proven

Every row in the advantages table becomes a benchmark scene before it becomes a claim on
the website. The rules are in
[`docs/performance/benchmark-contract.md`](../performance/benchmark-contract.md): public
scene, stated device, driver version, resolution, quality level, p50/p95/p99, and one
command that reproduces it.

Where a measurement shows an incumbent winning, we publish that too.
