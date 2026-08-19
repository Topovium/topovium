// SPDX-License-Identifier: GPL-3.0-or-later
//! The boundary between everything that describes rendering and everything that performs it.
//!
//! This crate names no graphics API. `wgpu` is the baseline implementation today and a
//! native Metal or DX12 fast path will be added later; when that happens, not one line
//! of scene, tool, or viewport code should change. That is the entire justification for
//! this crate existing, and it is why `AGENTS.md` §4.3 confines `wgpu` types to
//! `backends/wgpu-common`.
//!
//! If a type from a graphics library ever appears in this crate's public API, the
//! abstraction has already failed and the boundary check in CI will say so.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

mod capabilities;
mod device;

pub use capabilities::{BufferUsage, DeviceLimits, RenderFeature, SurfaceFormat};
pub use device::{BufferId, BufferWrite, RenderDevice, RenderError, TextureId};
