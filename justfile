# Topovium task runner.
#
# Every operation in this repository is a recipe here. If you find a step that
# isn't, that's a bug worth filing -- undocumented steps are how a project stops
# being contributable.

set shell := ["bash", "-uc"]

default:
    @just --list

# ---------------------------------------------------------------- verification

# Everything CI runs, in the order CI runs it. Run this before opening a PR.
check: fmt-check lint test boundaries deny
    @echo ""
    @echo "all checks passed"

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all --check

lint:
    cargo clippy --workspace --all-targets -- -D warnings

test:
    cargo test --workspace

# Enforce the layer contract in AGENTS.md section 4.3.
boundaries:
    cargo run -q -p xtask -- check-boundaries

# Licences, duplicate versions, and security advisories.
deny:
    cargo deny check

# ---------------------------------------------------------------------- running

run:
    cargo run -p topovium-desktop

run-release:
    cargo run --release -p topovium-desktop

# Print this machine's GPU capability report. Attach this to any bug report.
capability:
    cargo run -q -p topovium-cli -- capability-dump

scenes:
    cargo run -q -p topovium-cli -- list-scenes

# ---------------------------------------------------------------------- targets

web:
    pnpm --filter topovium-web dev

web-build:
    pnpm --filter topovium-web build

web-check:
    pnpm --filter topovium-web typecheck
    pnpm --filter topovium-web lint

# Cross-compile the core for Android. Requires the NDK and cargo-ndk.
android-build:
    cargo build -p topovium-ffi --target aarch64-linux-android --release

# Cross-compile the core for iPadOS.
ios-build:
    cargo build -p topovium-ffi --target aarch64-apple-ios --release

# Compile the viewport to WebAssembly for the browser target.
wasm-build:
    cargo build -p topovium-viewport --target wasm32-unknown-unknown --release

# --------------------------------------------------------------------- benchmarks

bench scene:
    cargo run --release -p topovium-cli -- list-scenes | grep -A 2 "{{scene}}"
    @echo "harness execution lands with the 0.0.1 benchmark milestone"

# ------------------------------------------------------------------------ misc

# Everything that must pass before a release.
ci: check web-check

clean:
    cargo clean
    rm -rf apps/web/.next apps/web/out node_modules
