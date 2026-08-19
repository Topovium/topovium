// SPDX-License-Identifier: GPL-3.0-or-later
//! The persistent GPU scene.
//!
//! The single most important performance decision in Topovium: scene data **lives** on
//! the GPU and is updated with deltas. The CPU never rebuilds or re-uploads the scene
//! to draw a frame.
//!
//! The difference is not incremental. Moving one object in a scene of a million:
//!
//! | Approach | Per-frame CPU work |
//! |---|---|
//! | Rebuild a snapshot each frame | 1,000,000 records copied |
//! | Persistent scene with deltas | 1 record written |
//!
//! Everything else — GPU culling, streaming, instancing — depends on this holding,
//! because none of them help if the CPU already walked every object to set it up.
//!
//! See `docs/rfc/0004-persistent-gpu-scene.md`.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

mod delta;
mod object_table;

pub use delta::{GpuSceneDelta, RangeUpdate};
pub use object_table::{ObjectFlags, ObjectRecord, ObjectTable};
