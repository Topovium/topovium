// SPDX-License-Identifier: GPL-3.0-or-later
//! The desktop editor.
//!
//! 0.0.1 scope: open a window, bring up a GPU device, and render at a measured,
//! budgeted frame rate. That is deliberately small. The frame budget manager, the
//! capability report, and the frame log are wired in from the very first frame, so
//! performance is something the project observes continuously rather than investigates
//! after someone complains.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

mod app;

use anyhow::Result;
use winit::event_loop::{ControlFlow, EventLoop};

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let event_loop = EventLoop::new()?;
    // Poll rather than Wait: the viewport redraws continuously while the frame budget
    // manager decides how much work each frame may contain.
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut application = app::Application::new();
    event_loop.run_app(&mut application)?;
    Ok(())
}
