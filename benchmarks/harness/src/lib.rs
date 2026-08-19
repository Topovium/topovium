// SPDX-License-Identifier: GPL-3.0-or-later
//! The benchmark harness.
//!
//! This exists before the features it measures, on purpose. A project whose entire
//! premise is speed and which adds measurement afterwards discovers its regressions
//! from users.
//!
//! Every result carries the machine it ran on and the command that reproduces it. A
//! number without those is not evidence, and `AGENTS.md` §5 does not allow publishing
//! one. The rules are in `docs/performance/benchmark-contract.md`.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

mod result;
mod scene;

pub use result::{BenchmarkResult, MemoryPeak, RegressionVerdict, TimingSummary, compare};
pub use scene::{BenchmarkScene, SceneCategory};
