# iPad shell

A thin Swift shell around the shared Rust core.

Its entire job is the five things Rust cannot portably do on iPadOS:

1. **Lifecycle** — scene phase, backgrounding, and the moment before the system may kill
   the process. The project is committed on `Backgrounded`, not on a timer.
2. **Surface** — a `CAMetalLayer` handed to the core, plus size and refresh-rate changes.
3. **Input** — Apple Pencil pressure, tilt, and hover; multi-touch; keyboard and trackpad.
4. **Documents** — the system file picker, security-scoped bookmarks, and drag and drop.
5. **Pressure signals** — thermal state and memory warnings, forwarded to the frame budget.

Everything else — the interface, the tools, the scene, the renderer — is Rust in
`crates/`. If this directory grows past roughly 500 lines, that is a design smell worth
an issue, not an excuse to write features here.

## Building

```sh
just ios-build                       # static library for aarch64-apple-ios
open apps/ipad-shell/Topovium.xcodeproj
```

The Xcode project is added in the 0.0.1 milestone issue "iPad shell: Metal surface and
lifecycle". Until then `TopoviumShell.swift` documents the contract and compiles nothing.
