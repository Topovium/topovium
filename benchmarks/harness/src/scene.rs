// SPDX-License-Identifier: GPL-3.0-or-later
use serde::{Deserialize, Serialize};

/// What aspect of the system a scene is designed to stress.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SceneCategory {
    Startup,
    Geometry,
    Instances,
    Materials,
    Lighting,
    Procedural,
    /// A long mobile session, measuring whether performance survives thermal throttling.
    MobileSustained,
}

/// A benchmark scene.
///
/// Every scene is public and generated from a seed rather than shipped as a binary
/// asset. That keeps the repository small and, more importantly, means anyone can
/// regenerate the exact scene a published number came from.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkScene {
    /// Stable identifier used in published results. Never reused for a different scene.
    pub id: String,
    pub category: SceneCategory,
    pub description: String,
    /// Seed for deterministic generation. The same seed must produce the same scene on
    /// every platform, or cross-platform results are not comparable.
    pub seed: u64,
    pub object_count: u64,
    pub triangle_count: u64,
    /// Command that reproduces this scene's run, verbatim.
    pub repro_command: String,
}

impl BenchmarkScene {
    /// The scenes 0.0.1 must be able to run.
    ///
    /// Small on purpose. A scene that nothing measures yet is a scene nobody maintains.
    #[must_use]
    pub fn baseline_suite() -> Vec<Self> {
        vec![
            Self {
                id: "startup-empty".to_owned(),
                category: SceneCategory::Startup,
                description: "Cold start to an editable viewport with an empty project.".to_owned(),
                seed: 0,
                object_count: 0,
                triangle_count: 0,
                repro_command: "just bench startup-empty".to_owned(),
            },
            Self {
                id: "instances-100k".to_owned(),
                category: SceneCategory::Instances,
                description: "100,000 instances of a single mesh, one material. Measures \
                              whether per-object CPU work has crept back in."
                    .to_owned(),
                seed: 1,
                object_count: 100_000,
                triangle_count: 100_000 * 12,
                repro_command: "just bench instances-100k".to_owned(),
            },
            Self {
                id: "transform-single-in-million".to_owned(),
                category: SceneCategory::Instances,
                description: "One object moved per frame in a scene of one million. The \
                              headline claim of the persistent GPU scene: this must cost \
                              one record, not a million."
                    .to_owned(),
                seed: 2,
                object_count: 1_000_000,
                triangle_count: 1_000_000 * 12,
                repro_command: "just bench transform-single-in-million".to_owned(),
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_scene_carries_a_reproduction_command() {
        // A published number nobody can reproduce is not evidence.
        for scene in BenchmarkScene::baseline_suite() {
            assert!(
                !scene.repro_command.is_empty(),
                "{} has no repro command",
                scene.id
            );
            assert!(
                !scene.description.is_empty(),
                "{} has no description",
                scene.id
            );
        }
    }

    #[test]
    fn scene_ids_are_unique() {
        let suite = BenchmarkScene::baseline_suite();
        let mut ids: Vec<&str> = suite.iter().map(|s| s.id.as_str()).collect();
        ids.sort_unstable();
        let count = ids.len();
        ids.dedup();
        assert_eq!(
            ids.len(),
            count,
            "scene ids must be unique and never reused"
        );
    }
}
