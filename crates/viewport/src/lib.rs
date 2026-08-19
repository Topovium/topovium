// SPDX-License-Identifier: GPL-3.0-or-later
//! Camera and frame budget.
//!
//! The frame budget manager is what turns "we target 60 FPS" into behaviour. Every
//! frame has a time budget derived from the display's refresh rate, and when the budget
//! is exceeded the renderer gives up quality in small, ordered steps rather than
//! dropping frames or falling off a preset cliff.
//!
//! The ordering is the design: the things users notice least go first.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

mod budget;
mod camera;

pub use budget::{FrameBudget, QualityLever, QualityState};
pub use camera::Camera;
