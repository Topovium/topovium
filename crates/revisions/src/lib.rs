// SPDX-License-Identifier: GPL-3.0-or-later
//! Revisions, dirty domains, and change sets.
//!
//! This crate is the vocabulary the whole system uses to answer one question: *what
//! actually changed?* Everything expensive in Topovium — evaluation, GPU upload,
//! path-tracer history, cache validity — is gated on that answer.
//!
//! It is a leaf crate. It knows nothing about scenes, GPUs, or files.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

mod change_set;
mod dirty;
mod revision;

pub use change_set::{ChangeSet, ChangedEntry};
pub use dirty::DirtyDomain;
pub use revision::{Revision, RevisionCounter};
