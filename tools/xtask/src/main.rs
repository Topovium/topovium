// SPDX-License-Identifier: GPL-3.0-or-later
//! Repository automation.
//!
//! Rules that live only in a document are rules that erode. Everything here turns a
//! line in `AGENTS.md` into a check that fails a build.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

mod boundaries;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "xtask", about = "Topovium repository automation")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Verify the architectural boundaries described in AGENTS.md section 4.3.
    CheckBoundaries,
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Command::CheckBoundaries => boundaries::check(),
    }
}
