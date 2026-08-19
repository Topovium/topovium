# Android shell

A thin Kotlin shell around the shared Rust core.

Its entire job is the five things Rust cannot portably do on Android:

1. **Lifecycle** — `GameActivity`, surface creation and destruction, and the moment
   before the process may be killed. The project is committed on `onStop`.
2. **Surface** — a `SurfaceView` handed to the core for Vulkan, plus size changes.
3. **Input** — S Pen pressure and tilt, multi-touch, keyboard, and mouse.
4. **Documents** — the Storage Access Framework and persisted URI permissions.
5. **Pressure signals** — the Android Dynamic Performance Framework's thermal headroom
   and performance hints, and `onTrimMemory`, forwarded to the frame budget.

Everything else is Rust in `crates/`.

## Why device capability is probed, never assumed

Android GPU fragmentation is not a detail to handle later. A device's Android version
tells you almost nothing about its Vulkan extensions, its limits, or its driver quality.
The core therefore classifies every device from measured features, and the shell's job is
to report what it finds rather than to guess from a model name.

## Building

```sh
just android-build                   # cdylib for aarch64-linux-android
./gradlew :app:assembleDebug
```

The Gradle project is added in the 0.0.1 milestone issue "Android shell: Vulkan surface
and lifecycle". Until then `TopoviumShell.kt` documents the contract and compiles nothing.
