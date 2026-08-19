// SPDX-License-Identifier: GPL-3.0-or-later
//! Core value types shared by every Topovium crate.
//!
//! `foundation` is a leaf: it depends on nothing else in this workspace, and nothing
//! in it knows about GPUs, windows, files, or user interfaces. See `AGENTS.md` §4.3.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

mod arena;
mod handle;
mod transform;
mod units;

pub use arena::{Arena, ArenaError};
pub use handle::{Generation, StableHandle};
pub use transform::Transform;
pub use units::{Meters, SceneScale};

/// Re-exported so every crate agrees on one linear algebra library and one set of
/// conventions. Topovium is right-handed, Y-up, metres.
pub mod math {
    pub use glam::{Affine3A, Mat3, Mat4, Quat, Vec2, Vec3, Vec3A, Vec4};

    /// The world up axis. Documented here so no code has to guess.
    pub const UP: Vec3 = Vec3::Y;
    /// The world forward axis in a right-handed, Y-up system.
    pub const FORWARD: Vec3 = Vec3::NEG_Z;
    /// The world right axis.
    pub const RIGHT: Vec3 = Vec3::X;

    /// View and projection matrices in Topovium's conventions.
    ///
    /// Right-handed, Y-up view space; NDC with Z in [0, 1] and Y up, which is what
    /// WebGPU, Metal, and DirectX all expect. Wrapped here rather than called directly
    /// so the convention is chosen once instead of at every call site, where getting it
    /// wrong produces an inverted or empty viewport rather than a compile error.
    pub mod camera {
        use glam::{Mat4, Vec3};

        /// A right-handed view matrix.
        #[must_use]
        pub fn look_at(eye: Vec3, target: Vec3, up: Vec3) -> Mat4 {
            glam::camera::rh::view::look_at_mat4(eye, target, up)
        }

        /// A reversed-Z perspective projection with an infinite far plane.
        ///
        /// Reversed-Z places floating-point precision where depth precision is scarce,
        /// which is what lets one scene span millimetres to kilometres without
        /// z-fighting or a hand-tuned near plane.
        #[must_use]
        pub fn perspective_infinite_reverse(fov_y: f32, aspect_ratio: f32, near: f32) -> Mat4 {
            glam::camera::rh::proj::directx::perspective_infinite_reverse(fov_y, aspect_ratio, near)
        }
    }
}
