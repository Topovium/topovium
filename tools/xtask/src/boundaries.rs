// SPDX-License-Identifier: GPL-3.0-or-later
//! Enforces the layer contract.
//!
//! An architecture diagram is a wish. This is the part that makes it true.

use anyhow::{Context, Result, bail};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// A rule of the form "files under `scope` must not mention `forbidden`".
struct Rule {
    scope: &'static str,
    forbidden: &'static [&'static str],
    reason: &'static str,
    /// Paths under `scope` that are exempt, and why.
    exceptions: &'static [&'static str],
}

const RULES: &[Rule] = &[
    Rule {
        scope: "crates",
        forbidden: &["wgpu::", "use wgpu", "wgpu ="],
        reason: "wgpu types belong only in backends/wgpu-common. Naming them here would \
                 make a native Metal or DX12 fast path impossible to add without \
                 rewriting scene code.",
        exceptions: &[],
    },
    Rule {
        scope: "crates/platform-api",
        forbidden: &["winit", "UIKit", "jni::", "web_sys"],
        reason: "platform-api describes what a shell must provide without naming any \
                 platform. A concrete platform type here defeats the abstraction.",
        exceptions: &[],
    },
    Rule {
        scope: "crates/foundation",
        forbidden: &["topovium_"],
        reason: "foundation is a leaf crate and must not depend on anything else in \
                 the workspace.",
        exceptions: &[],
    },
    Rule {
        scope: "crates/revisions",
        forbidden: &["topovium_"],
        reason: "revisions is a leaf crate and must not depend on anything else in \
                 the workspace.",
        exceptions: &[],
    },
    Rule {
        scope: "crates",
        forbidden: &["Arc<Mutex<"],
        reason: "A lock per scene object means a million allocations and a million \
                 locks in a large scene. Use handles into columnar storage.",
        exceptions: &[
            // The job pool's shared queue is one lock for the whole pool, which is the
            // opposite of the per-object pattern this rule targets.
            "crates/job-system/src/pool.rs",
        ],
    },
];

/// Runs every rule, reporting all violations rather than stopping at the first.
///
/// # Errors
/// Returns an error listing every violation found.
pub fn check() -> Result<()> {
    let root = repository_root()?;
    let mut violations = String::new();
    let mut count = 0usize;

    for rule in RULES {
        let scope = root.join(rule.scope);
        if !scope.exists() {
            continue;
        }
        for file in rust_files(&scope)? {
            let relative = file
                .strip_prefix(&root)
                .unwrap_or(&file)
                .to_string_lossy()
                .replace('\\', "/");
            if rule.exceptions.iter().any(|e| relative == *e) {
                continue;
            }
            // A crate may always refer to itself; skip the crate that owns the rule's
            // forbidden name where that is the point of the crate.
            let contents = std::fs::read_to_string(&file)
                .with_context(|| format!("reading {}", file.display()))?;

            for (line_number, line) in contents.lines().enumerate() {
                let code = line.split("//").next().unwrap_or(line);
                for needle in rule.forbidden {
                    if code.contains(needle) {
                        count += 1;
                        let _ = writeln!(
                            violations,
                            "  {}:{}\n    contains {needle:?}\n    {}",
                            relative,
                            line_number + 1,
                            rule.reason
                        );
                    }
                }
            }
        }
    }

    if count > 0 {
        bail!("{count} architectural boundary violation(s):\n{violations}");
    }
    println!("boundary check passed: {} rules, 0 violations", RULES.len());
    Ok(())
}

fn repository_root() -> Result<PathBuf> {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").context("CARGO_MANIFEST_DIR is not set")?;
    Path::new(&manifest)
        .ancestors()
        .find(|candidate| candidate.join("AGENTS.md").exists())
        .map(Path::to_path_buf)
        .context("could not locate the repository root (no AGENTS.md found in any ancestor)")
}

fn rust_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut found = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(directory) = stack.pop() {
        for entry in std::fs::read_dir(&directory)
            .with_context(|| format!("reading {}", directory.display()))?
        {
            let path = entry?.path();
            if path.is_dir() {
                if path.file_name().is_some_and(|name| name == "target") {
                    continue;
                }
                stack.push(path);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                found.push(path);
            }
        }
    }
    found.sort();
    Ok(found)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_workspace_currently_satisfies_every_boundary_rule() {
        // If this fails, either the code broke a rule or a rule needs to change --
        // deliberately, in AGENTS.md, not by deleting the check.
        check().unwrap();
    }

    #[test]
    fn every_rule_explains_itself() {
        for rule in RULES {
            assert!(
                !rule.reason.is_empty(),
                "scope {} has no reason",
                rule.scope
            );
            assert!(!rule.forbidden.is_empty());
        }
    }
}
