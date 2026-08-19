# Contributing to Topovium

Read [AGENTS.md](./AGENTS.md) first. It is short, binding, and applies to humans and AI
assistants equally.

## Getting set up

```sh
git clone https://github.com/Topovium/topovium
cd topovium
just check      # everything CI runs, in the order CI runs it
just run        # the desktop viewport
```

`rustup` installs the pinned toolchain automatically. The website needs Node 22 and pnpm.

`just --list` shows every task. There is no operation in this project that is not a
recipe there; if you find one, that is a bug worth filing.

## The workflow

1. **Open an issue first** for anything beyond a typo. It is much cheaper to disagree
   about an approach in an issue than in a finished branch.
2. **Branch from `main`.** `main` is protected; nobody pushes to it directly.
3. **Write the test first** for a bug fix. A fix without a test that failed before it is
   a fix nobody can prove works, or prove stays working.
4. **Run `just check`** before opening the pull request.
5. **Fill in the pull request template honestly**, including what is not covered.

## What gets a change rejected

None of these are stylistic:

- A secret, key, token, or `.env` file. This repository is public and history is forever.
- `any` in TypeScript, or `unwrap`/`expect` outside tests in Rust.
- `unsafe` without an accepted ADR and a `// SAFETY:` comment per block.
- A `wgpu` type outside `backends/wgpu-common`.
- `Arc<Mutex<_>>` per scene object, a global scene lock, or a full-scene copy per frame.
- Blocking the UI thread on save, shader compilation, BVH build, or import.
- A performance claim with no benchmark scene behind it.
- Generated code that no human has read line by line.

## Working with AI assistants

Topovium is built with heavy AI assistance, deliberately. It stays safe because the line
between proposed and accepted is absolute:

- Assistants write files and explain them. **A human commits.**
- Assistants never push, tag, merge, or touch `main`. Branch protection enforces this
  independently of anyone's good intentions.
- Six areas require a human-authored ADR and are never decided by an assistant alone:
  memory model, GPU synchronisation, project format, `unsafe`, procedural topology, and
  recovery policy.

If you use an assistant, you are the author of what it produced and responsible for
understanding it. "The model wrote it" is not a review.

## Design decisions

Anything expensive to reverse gets written down before it is built:

- **ADR** (`docs/adr/`) — a decision that has been made, with its consequences.
- **RFC** (`docs/rfc/`) — a proposal still under discussion.

Use the RFC issue template to start one.

## Licence

Contributions are licensed under GPL-3.0-or-later. By opening a pull request you confirm
you have the right to contribute the code under that licence.
