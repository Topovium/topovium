# RFCs

An RFC is written before building anything expensive to reverse. AGENTS.md §1.3 lists the
areas where one is mandatory: memory model, GPU synchronisation, project format, `unsafe`,
procedural topology, and recovery policy.

An RFC that has been accepted becomes an ADR in `docs/adr/`.

| RFC | Subject | Status |
|---|---|---|
| 0001 | Product wedge and non-goals | planned |
| 0002 | Platform capability contract | planned |
| 0003 | Revisions, handles, and change sets | partly built; write-up owed |
| **0004** | **Persistent GPU scene** | accepted as [ADR-0002](../adr/0002-persistent-gpu-scene.md) |
| 0005 | Frame budget and mobile thermal policy | partly built; write-up owed |
| 0006 | Editable mesh, lineage, and remapping | planned |
| 0007 | Render API and native fast paths | baseline accepted as [ADR-0001](../adr/0001-rust-with-wgpu-baseline.md) |
| 0008 | Transactional project format | planned — the highest-stakes remaining decision |
| 0009 | Benchmark harness | partly built; contract in `docs/performance/` |
| 0010 | Tool runtime and cross-device input | planned |

RFC-0004 came first because the viewport, streaming, selection, and the renderer all
depend on it. RFC-0008 is next in stakes: getting the project format wrong means users
lose work permanently, and it is the one decision that cannot be fixed in a later release.
