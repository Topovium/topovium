# ADR-0003: AI assistants propose diffs; humans commit

- **Status:** accepted
- **Date:** 2026-08-20

## Context

Topovium is built with heavy AI assistance. That is a deliberate choice: it is what makes
a project of this scope tractable for a small team. It also introduces a specific failure
mode.

Generated code that is subtly wrong about memory ordering, GPU synchronisation, or a
numerical edge case compiles perfectly and passes shallow tests. Compilation checks types.
It does not check truth. Without a hard boundary, plausible-looking code accumulates faster
than anyone can read it, and the project acquires a codebase nobody understands.

## Decision

AI assistants never run `git commit`, `git push`, `git tag`, or `gh pr merge`, and never
write to `main`. They write files and explain them; a human reads the diff and commits.

Branch protection on `main` enforces this independently, because a rule that relies only on
good intentions is not a rule.

Six areas additionally require a human-authored ADR: memory model, GPU synchronisation,
project format, `unsafe`, procedural topology semantics, and recovery policy. An assistant
may draft one; a human accepts it.

## Consequences

**Good.** Every line in the repository has been read by a person who could explain it. The
six reserved areas are exactly those where a wrong decision means data loss, undefined
behaviour, or a bug that reproduces on one vendor's driver — not a failing test.

**Bad.** Slower than letting assistants commit directly. Requires the human to actually
read the diff rather than skim it, which is a discipline no tool can enforce.

**Rejected: assistants commit to a branch, humans review the pull request.** Superficially
similar, but review attention drops sharply once code is already committed and CI is green.
The friction of the human doing the commit is the point, not an accident.
