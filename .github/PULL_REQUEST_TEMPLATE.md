## What changed

<!-- One or two sentences. What does this do that the codebase could not do before? -->

## Why

<!-- The problem, not the solution. Link the issue if there is one. -->

## How it was verified

<!-- Commands you actually ran, and their result. "Should work" is not verification. -->

```
just check
```

## What is not covered

<!-- Be specific. An honest gap is useful; a silent one is a trap for the next person. -->

---

### Checklist

Tick only what you verified. A ticked box you did not check is worse than an unticked
one, because it turns an open question into a false answer.

- [ ] `just check` passes locally
- [ ] No secrets, keys, tokens, or `.env` files added — this repository is public
- [ ] No `any`, `@ts-ignore`, or non-null assertions in TypeScript
- [ ] No `unwrap`/`expect` outside tests; no new `unsafe` without an accepted ADR
- [ ] No `Arc<Mutex<_>>` per scene object, no global scene lock, no full-scene copy per frame
- [ ] Nothing added that blocks the UI thread on save, shader compilation, BVH, or import
- [ ] Behaviour is covered by a test; a bug fix starts with a test that reproduced it
- [ ] Any performance claim has a benchmark scene backing it
- [ ] A human read every line of this diff, including any that was AI-generated
