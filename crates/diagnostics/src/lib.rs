// SPDX-License-Identifier: GPL-3.0-or-later
//! Measurement.
//!
//! Topovium's premise is being measurably faster, which obligates it to measure. This
//! crate is how: frame timing with percentiles, stall detection, and a capability
//! report that says exactly what hardware a number came from.
//!
//! Percentiles, never averages. An average frame time of 8 ms hides the 90 ms hitch
//! that is the only thing the user actually noticed.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

mod capability;
mod frame_log;

pub use capability::{Backend, CapabilityReport, DeviceTier};
pub use frame_log::{FrameLog, FrameSample, Percentiles};
