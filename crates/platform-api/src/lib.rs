// SPDX-License-Identifier: GPL-3.0-or-later
//! What a platform shell must provide, expressed without naming any platform.
//!
//! Swift, Kotlin, `winit`, and the browser each implement these traits. Nothing above
//! this crate knows which one is running, which is what lets the same tool code handle
//! a mouse, an Apple Pencil, an S Pen, and a touch screen without branching on OS.
//!
//! Nothing here may mention `UIKit`, JNI, the DOM, or `winit`. See `AGENTS.md` §4.3.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

mod input;
mod lifecycle;

pub use input::{ButtonState, InputEvent, PointerId, PointerKind, PointerSample};
pub use lifecycle::{LifecycleEvent, MemoryPressure, ThermalState};

/// A drawable surface owned by the shell.
///
/// The shell owns the window or view; the renderer only ever borrows its size and is
/// told when it changes. Ownership sitting on the platform side is what keeps
/// lifecycle handling native and correct on iPadOS and Android.
pub trait Surface: std::fmt::Debug {
    /// Current drawable size in physical pixels.
    fn size(&self) -> (u32, u32);

    /// Ratio of physical pixels to logical points.
    fn scale_factor(&self) -> f32;

    /// The display's refresh rate, where the platform reports it.
    ///
    /// Drives the frame budget: 120 Hz is an 8.33 ms budget, 60 Hz is 16.67 ms.
    /// `None` means fall back to a conservative 60 Hz assumption.
    fn refresh_rate_hz(&self) -> Option<f32>;
}
