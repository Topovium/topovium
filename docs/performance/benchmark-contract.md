# Benchmark contract

Topovium's entire premise is being measurably faster. That obligates us to measure
honestly, including when the result is unflattering.

These rules apply to our own results, not only to comparisons.

## Every published result carries

- Scene id, and the seed that generates it
- Device, operating system, and driver version
- Backend (Vulkan, Metal, DX12, WebGPU)
- Resolution and quality level
- p50, p95, p99, and the worst frame
- Peak CPU, GPU, and shared memory
- Number and duration of stalls
- The exact command that reproduces it

A result missing any of these is not published.

## Rules

**Percentiles, never averages.** A mean frame time of 8 ms hides the 90 ms hitch that is
the only thing the user noticed. Averages are not reported at all.

**Nearest-rank percentiles.** A reported p99 must be a frame time that actually occurred,
so it can be traced to a specific frame in a capture. Interpolated percentiles report
numbers no frame ever took.

**Public scenes.** Every scene is generated from a seed rather than shipped as a binary,
so anyone can regenerate exactly what we measured.

**Identical conditions.** Same hardware, same resolution, same quality level. The
comparison tool refuses to compare runs that differ in any of these rather than silently
treating the difference as a result.

**A faster run at lower quality is not a faster run.** This is the most common way a
benchmark lies, and the harness rejects it structurally.

**No cherry-picking.** Results come from full runs, not the best of several.

**Regressions are published.** Including cases where another application wins. A page
that only ever shows us winning is marketing, and nobody believes marketing.

## The merge gate

A pull request is blocked when, on a reference scene:

- p95 or p99 rises more than 5% above the baseline
- peak memory rises past its threshold
- a new stall appears (any frame over 50 ms)
- startup time regresses
- an image changes without an approved golden-image update
- sustained mobile performance degrades

Five percent is above measurement noise on a quiet machine and below what a user would
notice, so it catches real drift without failing builds on jitter.

## Comparing against other applications

A comparison is published only when all of these hold:

- The scene is public and both applications can load it
- The hardware and operating system are identical
- Resolution and image quality are stated and equivalent
- The test is automated and runs unattended
- p50, p95, and p99 are reported for both
- Memory is reported for both
- Driver versions are recorded
- The whole thing reproduces from one command

Without every one of these, "faster" is a slogan.

## Targets

Design gates, not marketing promises. Each becomes a merge-blocking check as the feature
it measures lands.

### Reference desktop

| Metric | Target |
|---|---|
| Cold start to editable viewport | ≤ 2.0 s |
| Warm start | ≤ 0.8 s |
| Empty project memory | ≤ 350 MB |
| CPU with an idle scene | < 1% of one core |
| Longest UI stall while modelling | < 50 ms |
| Input-to-pixel p95 at 120 Hz | ≤ 25 ms |
| Large-instance scene | 60 FPS p95 |
| Useful denoised preview | ≤ 1.0 s |

### Reference iPad

| Metric | Target |
|---|---|
| Cold start | ≤ 3.0 s |
| Warm start | ≤ 1.5 s |
| Sustained frame rate | 60 FPS for 20 minutes |
| Input-to-pixel p95, stylus | ≤ 33 ms |
| Project loss after the system kills the app | none |

### Reference Android

| Metric | Target |
|---|---|
| Frame pacing | stable, not maximum |
| Sustained session | 20 minutes with a thermal headroom report |
| Driver fallback | controlled, never a black screen |
