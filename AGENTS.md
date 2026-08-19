# AGENTS.md — Golden Rules

Rules for every contributor to Topovium, human or AI. They are not style preferences.
Each one exists because breaking it has a known, concrete cost to this project.

Where a rule can be checked by a machine, it is. Where it cannot, it is a line in the
pull request template that a human ticks and is accountable for.

**Precedence:** this file > skills and tooling defaults > model habits. If an
instruction here conflicts with a default behaviour, this file wins.

---

## 0. The one-paragraph version

Never commit. Never push. Never write a secret. Never write `any`. Never write
`unsafe` without an ADR. Never claim something is faster without a benchmark that
proves it. Never accept code because it compiles.

---

## 1. Agent conduct

Topovium is built with heavy AI assistance. That is a deliberate choice, and it only
stays safe if the boundary between "proposed" and "accepted" is absolute.

### 1.1 Agents do not commit

Agents **must not** run:

- `git commit`, `git commit --amend`
- `git push`, `git push --force`, any force-push in any form
- `git tag`, `git rebase`, `git reset --hard`
- `gh pr merge`, `gh release create`
- any command that writes to `main`

Agents write files and explain them. A human reads the diff and commits it. Branch
protection on `main` enforces this independently of anyone's good intentions, because
a rule that depends only on good intentions is not a rule.

If an agent believes a commit is needed, it says so and stops.

### 1.2 Compiling is not a review

Code is accepted because a human understood it, not because `cargo build` exited 0.
A generated file that nobody has read line by line does not merge, no matter how
clean the CI run looks.

The specific failure this prevents: plausible code that is subtly wrong about memory
ordering, GPU synchronisation, or numerical edge cases compiles perfectly and passes
shallow tests. Compilation checks types. It does not check truth.

### 1.3 Decisions agents may not make alone

These require a human-authored, human-accepted ADR in `docs/adr/`:

| Area | Why it is reserved |
|---|---|
| Memory model and residency policy | Wrong here means OOM crashes on user machines, not test failures |
| GPU synchronisation and resource lifetime | Wrong here means driver hangs and corrupted frames that reproduce on one vendor only |
| Project file format and migrations | Wrong here means users lose work permanently |
| Any use of `unsafe` | Wrong here means undefined behaviour, not an error message |
| Procedural topology and element lineage | Wrong here silently breaks every downstream operation in a user's history |
| Crash recovery policy | Wrong here destroys the file it was meant to save |

An agent may draft the ADR. A human accepts it.

### 1.4 Scope

An agent does the task asked. It does not opportunistically refactor adjacent code,
rename things it finds ugly, upgrade dependencies, or "improve" files it was not asked
to touch. Unrequested changes hide the requested ones from review.

### 1.5 Honest reporting

If tests fail, say so and paste the output. If a step was skipped, say which and why.
If something is unverified, call it unverified. Never report a task as complete
without having run the check that proves it.

---

## 2. Secrets — this repository is public

Everything here is world-readable, permanently, including history. `git rm` does not
delete anything; it adds a commit.

### 2.1 Never committed, in any branch, ever

API keys, tokens, passwords, private keys, `.env` files of any kind, TLS certificates,
Apple provisioning profiles, `.p12` bundles, Android keystores, service-account JSON,
session cookies, database URLs containing credentials, personal email addresses,
internal hostnames, or signed URLs.

### 2.2 Where they live instead

GitHub Actions secrets, referenced by name in workflows. Nowhere else in this repo.

### 2.3 Enforcement

- `gitleaks` runs on every push and every pull request and blocks the merge.
- `.gitignore` covers `.env*`, `*.pem`, `*.key`, `*.p12`, `*.mobileprovision`,
  `*.keystore`, `*.jks`.
- Example configuration files are committed as `*.example` with placeholder values
  that are obviously fake.

### 2.4 If a secret is committed

**Rotate it first. Scrub the history second.** Never the other way round. The moment a
credential is pushed to a public repository it must be treated as compromised — scrapers
index public pushes within seconds. Cleaning history on a credential you have not yet
revoked accomplishes nothing except a false sense of safety.

Then open a `type:bug` `risk:irreversible` issue recording what leaked, when it was
rotated, and what change prevents a repeat.

---

## 3. TypeScript

Applies to `apps/web` and every future TypeScript surface.

### 3.1 Banned outright

- **`any`.** `@typescript-eslint/no-explicit-any` is set to `error`. There is no
  approved use of `any` in this codebase. If a type is genuinely unknown, it is
  `unknown` and you narrow it.
- **`@ts-ignore`.** Silences the compiler without recording why.
- **`@ts-expect-error`** outside test fixtures, and there it needs a description.
- **Non-null assertion `!`** on a value that can actually be null. Handle the null.
- **Implicit `any`** — `noImplicitAny` stays on.

### 3.2 Required compiler settings

`strict`, `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`,
`noFallthroughCasesInSwitch`, `noImplicitOverride`, `verbatimModuleSyntax`.

`noUncheckedIndexedAccess` in particular is non-negotiable: without it,
`array[i]` claims a value exists when it does not, and the resulting crash lands in
production rather than in `tsc`.

### 3.3 Rules with judgement

- `unknown` must be narrowed before it crosses a module boundary. Callers should never
  receive a value they have to guess about.
- A type assertion (`as`) requires a comment on the line above stating what the
  compiler cannot know and why you can. `as const` and `as unknown as T` in tests are
  exempt.
- Prefer discriminated unions over optional fields that are only valid in combination.
  If two fields must be set together, the type should make that impossible to get wrong.
- No default exports except where a framework requires them (Next.js pages, layouts).
  Named exports survive renames and are greppable.
- Runtime data — API responses, JSON files, `localStorage`, WASM boundaries — is parsed
  and validated at the boundary, not cast. A type assertion on untrusted input is a lie
  told to the compiler.

---

## 4. Rust

### 4.1 Safety

- Every crate carries `#![forbid(unsafe_code)]` by default.
- The exceptions are `crates/ffi` and `backends/*`, which need `#![deny(unsafe_op_in_unsafe_fn)]`
  instead, an accepted ADR, and a `// SAFETY:` comment on every `unsafe` block stating
  the invariant being upheld and who upholds it.
- `unwrap()` and `expect()` are denied outside `#[cfg(test)]`. A panic in a DCC is a
  user's unsaved work.
- `panic!` in library code is a bug. Return an error.
- No `std::process::exit` outside `apps/*/src/main.rs`.

### 4.2 Lints

`cargo clippy --workspace --all-targets -- -D warnings` must pass. Warnings are not
advisory here; an ignored warning list becomes an ignored error list.

### 4.3 Architecture rules

These come from the design and are enforced by `cargo xtask check-boundaries`:

- **`wgpu` types appear in exactly one place:** `backends/wgpu-common`. Not in scene
  code, not in tools, not in `render-api`'s public surface. The whole point of the
  abstraction is that adding a native Metal or DX12 fast path later does not require
  touching a single line of scene code.
- **No `Arc<Mutex<T>>` per scene object.** A million objects means a million locks and
  a million allocations. Use handles into columnar storage.
- **No global scene lock.** It serialises exactly the work that needed to be parallel.
- **No full-scene copy per frame.** Send deltas. Cost must scale with what changed and
  what is visible, not with total scene size.
- **The UI thread never blocks** on: save, shader compilation, BVH build, import,
  mesh rebuild, or network. If it can take 16 ms, it goes to the job system.
- **No CPU loop over all objects** as a shipped architecture. Culling and LOD selection
  belong on the GPU.
- **Layers depend downward only.** `foundation`, `revisions`, `job-system`, and
  `platform-api` are leaves and depend on nothing in this workspace.

### 4.4 Errors

`thiserror` in libraries, `anyhow` in binaries. Error types say what failed and what
the caller can do about it. `Box<dyn Error>` in a library signature is not an error
type, it is a refusal to design one.

---

## 5. Performance

Topovium's entire premise is being measurably faster. That obligates us to measure.

- **A performance claim without a benchmark scene does not merge.** Not in code
  comments, not in the README, not in a commit message, not on the website.
- Benchmarks report **p50, p95 and p99** — never a bare average. An average hides
  exactly the stalls users feel.
- Every published result states: scene, device, OS, driver version, backend,
  resolution, quality tier, memory peak, and the command to reproduce it.
- Optimisation without a trace and a reproducing scene is guessing. Profile first.
- We publish regressions and we publish the cases where competitors win. A benchmark
  page that only ever shows us winning is marketing, and nobody believes marketing.

The full contract is in `docs/performance/benchmark-contract.md`.

---

## 6. Testing

- Behaviour is tested, not implementation. A test that breaks on every refactor is a
  cost with no benefit.
- Geometry and serialisation get property-based tests. Hand-picked examples do not
  find the winding-order bug; generated ones do.
- Every file format version gets a migration test from every previous version, with a
  committed fixture. A user must be able to open a project made a year ago.
- Importers get fuzzed. They parse untrusted input from the internet.
- A bug fix starts with a failing test that reproduces it.

---

## 7. Dependencies

- Every dependency is a permanent commitment and a supply-chain surface. Adding one is
  a decision, not a convenience.
- Critical dependencies need a documented exit plan: the wrapper boundary, the data
  format at that boundary, the licence, supported platforms, binary cost, and the
  fallback if the project is abandoned.
- `cargo deny check` gates licences, duplicate versions, and security advisories.
- We do not outsource our competitive advantage. The command model, dependency graph
  policy, persistent GPU scene, residency manager, frame budget, and tool runtime stay
  ours.

---

## 8. Pull requests

Every PR states what changed, why, how it was verified, and what was not covered.

The checklist in `.github/PULL_REQUEST_TEMPLATE.md` mirrors this file. Ticking a box
you did not verify is worse than leaving it unticked, because it converts an open
question into a false answer.

---

## 9. When a rule is wrong

Rules that cannot be questioned rot. If one of these is blocking correct work, open an
issue arguing the case and change it here. What is not acceptable is quietly ignoring
it — the next person reads the file and believes it.
