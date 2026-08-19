# Security policy

## Reporting a vulnerability

Report privately through
[GitHub's security advisory form](https://github.com/Topovium/topovium/security/advisories/new).
Please do not open a public issue for a vulnerability.

Expect an acknowledgement within seven days.

## What is in scope

Topovium is a local-first desktop and mobile application with no server component, so the
interesting attack surface is the data it opens:

- **Importers and the project format.** These parse untrusted files from the internet.
  Memory-safety bugs, panics on malformed input, and path traversal during import all
  count.
- **The plugin host.** Sandbox escapes and anything that lets a plugin reach outside its
  declared capabilities.
- **The FFI boundary.** Undefined behaviour reachable from a platform shell.
- **Supply chain.** A compromised or typosquatted dependency in `Cargo.toml` or
  `package.json`.

## What is not in scope

- The website is a static export with no backend and no user data.
- Denial of service caused by opening a deliberately enormous scene. The memory budget
  degrades quality and reports a clear error; that is intended behaviour, not a bug.
- Vulnerabilities in a dependency that are already public and tracked in `deny.toml`.

## If a credential leaks into this repository

**Rotate it first. Scrub history second.** Never the other way round.

The moment a credential is pushed to a public repository it must be treated as
compromised — scrapers index public pushes within seconds. Cleaning history on a
credential that has not been revoked accomplishes nothing except a false sense of safety.

Then open a `type:bug` `risk:irreversible` issue recording what leaked, when it was
rotated, and what change prevents a repeat.
