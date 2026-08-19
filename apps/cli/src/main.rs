// SPDX-License-Identifier: GPL-3.0-or-later
//! Topovium's command line tools.
//!
//! `capability-dump` is the first thing to run on a new machine and the first thing to
//! attach to a bug report. Almost every graphics problem is really "this device does
//! not have what the code assumed", and this prints exactly what it does have.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use topovium_bench_harness::BenchmarkScene;
use topovium_wgpu_common::describe_adapter;

#[derive(Parser, Debug)]
#[command(name = "topovium", version, about = "Topovium command line tools")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Print this machine's GPU capability report as JSON.
    CapabilityDump,
    /// List the benchmark scenes this build can run.
    ListScenes,
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    match Cli::parse().command {
        Command::CapabilityDump => capability_dump(),
        Command::ListScenes => list_scenes(),
    }
}

fn capability_dump() -> Result<()> {
    let instance =
        wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle_from_env());

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: None,
        ..Default::default()
    }))
    .context(
        "no graphics adapter found. On Linux this usually means Vulkan drivers are not \
         installed; on a virtual machine, that GPU passthrough is unavailable.",
    )?;

    let report = describe_adapter(
        &adapter.get_info(),
        adapter.features(),
        &adapter.limits(),
        env!("CARGO_PKG_VERSION"),
    );

    println!(
        "{}",
        report
            .to_json()
            .context("serialising the capability report")?
    );
    Ok(())
}

fn list_scenes() -> Result<()> {
    for scene in BenchmarkScene::baseline_suite() {
        println!("{:<32} {:?}", scene.id, scene.category);
        println!("    {}", scene.description);
        println!("    reproduce: {}\n", scene.repro_command);
    }
    Ok(())
}
