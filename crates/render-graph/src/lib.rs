// SPDX-License-Identifier: GPL-3.0-or-later
//! A frame declared as passes and the resources they read and write.
//!
//! Two things fall out of declaring a frame instead of imperatively issuing it:
//!
//! **Ordering is derived, not hand-maintained.** Passes are sorted by their data
//! dependencies, so inserting a pass cannot break the barrier placement of another.
//!
//! **Transient memory is reused.** The graph knows each temporary's lifetime, so the
//! ambient-occlusion, bloom, and denoiser targets that never coexist can share pages.
//! On a tablet with unified memory, that reuse is the difference between fitting and
//! being killed by the operating system.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

mod aliasing;
mod graph;

pub use aliasing::{AliasPlan, ResourceLifetime, plan_aliasing};
pub use graph::{GraphError, PassId, RenderGraph, ResourceId};
