// SPDX-License-Identifier: GPL-3.0-or-later
//! The `wgpu` implementation of [`topovium_render_api::RenderDevice`].
//!
//! **This is the only crate in the workspace allowed to name a `wgpu` type.** CI
//! enforces it (`cargo xtask check-boundaries`), and the reason is concrete: a native
//! Metal or DX12 fast path is planned, and when it arrives the scene, tool, and
//! viewport code must not change by a single line to accommodate it.
//!
//! `wgpu` is the portable baseline, not the ceiling. Anything it cannot express yet —
//! hardware ray tracing, mesh shaders, sparse residency — is reachable through a
//! second implementation of the same trait.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

mod capability;
mod device;

pub use capability::{describe_adapter, map_features, map_limits};
pub use device::WgpuDevice;
