# Versioning

Topovium versions are `MAJOR.MINOR.PATCH`, and the patch component runs the full range
before the minor rolls over.

```
0.0.1 … 0.0.99   →   0.1.0 … 0.1.99   →   0.2.0 … 0.2.99   →   …   →   1.0.0
```

So `0.0.99` is followed by `0.1.0`, never by `0.0.100` and never by `0.1.1`.

## What each component means

**Patch** — one milestone of work. Each has a gate: a condition someone else can check.
A patch version is released when its gate passes, not when the work feels finished. Up to
99 of them fit inside a minor, which is deliberate headroom: it means a milestone can stay
small and specific rather than being padded to justify a version bump.

**Minor** — a step in what the product can do. `0.1.0` is the first public alpha: the
point at which someone outside the project can install it and produce something.

**Major** — `1.0.0` is reserved for the point at which Topovium is the best available
choice for modelling, large-scene assembly, look-development, and cross-device work, with
published benchmarks backing that claim. It is not a date.

## Rules

- A version is never reused or retracted once published.
- The project format version is independent of the application version. Every application
  version must open every project format version ever released, with a committed migration
  fixture proving it.
- A version number is not a marketing decision. `0.0.7` following `0.0.6` says one
  milestone gate passed, nothing more.

## Why not semantic versioning yet

Below `1.0.0` there is no stable public API to make promises about, so the semantic
versioning contract has nothing to attach to. Once `1.0.0` ships, breaking changes to the
plugin API and the project format follow semver properly. Until then the version is a
progress marker, and this document is what stops it from being read as more than that.
