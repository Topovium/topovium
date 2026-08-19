// SPDX-License-Identifier: GPL-3.0-or-later
use serde::{Deserialize, Serialize};
use topovium_diagnostics::CapabilityReport;

/// Percentiles for one measured quantity, in milliseconds.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimingSummary {
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub max_ms: f64,
    pub sample_count: usize,
}

/// Peak memory during a run.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryPeak {
    pub cpu_bytes: u64,
    /// `None` on unified-memory devices, where a separate figure would double-count.
    pub gpu_bytes: Option<u64>,
    pub shared_bytes: Option<u64>,
}

/// One benchmark run.
///
/// Published verbatim to the website. Everything needed to judge or reproduce the
/// number is in here; nothing is left to a caption.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkResult {
    pub schema_version: u32,
    pub scene_id: String,
    /// Commit the measurement was taken at.
    pub commit: String,
    /// ISO 8601, supplied by the caller so the harness stays deterministic.
    pub timestamp: String,
    pub capability: CapabilityReport,
    pub resolution: [u32; 2],
    /// Quality tier as reported by the frame budget manager. Comparing runs at
    /// different quality levels is the most common way benchmarks mislead, so it is
    /// recorded rather than assumed.
    pub quality_levers_engaged: u8,
    pub cpu_frame: TimingSummary,
    pub gpu_frame: Option<TimingSummary>,
    pub input_to_pixel: Option<TimingSummary>,
    pub memory_peak: MemoryPeak,
    /// Frames exceeding the 50 ms stall threshold.
    pub stall_count: u32,
    pub repro_command: String,
}

impl BenchmarkResult {
    /// Current schema version.
    pub const SCHEMA_VERSION: u32 = 1;

    /// Serialises for publication.
    ///
    /// # Errors
    /// Returns an error if serialisation fails.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

/// Whether a run is acceptable compared with a baseline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegressionVerdict {
    /// Within tolerance.
    Pass,
    /// Measurably better. Reported so improvements are recorded, not just failures.
    Improved { detail: String },
    /// Beyond tolerance. Blocks the merge.
    Regressed { detail: String },
    /// Not comparable — different scene, device, or resolution. Never silently treated
    /// as a pass, because a benchmark that quietly stops comparing is worse than none.
    Incomparable { reason: String },
}

/// Tolerance before a change counts as a regression.
///
/// Five percent is above measurement noise on a quiet machine and below what a user
/// would notice, so it catches real drift without failing builds on jitter.
const TOLERANCE: f64 = 0.05;

/// Compares a run against a baseline, on p95 and p99.
///
/// Not the average. A change that leaves the mean untouched while doubling p99 has
/// made the application feel worse, and that is exactly the change this gate exists
/// to catch.
#[must_use]
pub fn compare(baseline: &BenchmarkResult, candidate: &BenchmarkResult) -> RegressionVerdict {
    if baseline.scene_id != candidate.scene_id {
        return RegressionVerdict::Incomparable {
            reason: format!(
                "different scenes: {} and {}",
                baseline.scene_id, candidate.scene_id
            ),
        };
    }
    if baseline.resolution != candidate.resolution {
        return RegressionVerdict::Incomparable {
            reason: format!(
                "different resolutions: {:?} and {:?}",
                baseline.resolution, candidate.resolution
            ),
        };
    }
    if baseline.capability.adapter_name != candidate.capability.adapter_name {
        return RegressionVerdict::Incomparable {
            reason: format!(
                "different adapters: {} and {}",
                baseline.capability.adapter_name, candidate.capability.adapter_name
            ),
        };
    }
    if baseline.quality_levers_engaged != candidate.quality_levers_engaged {
        return RegressionVerdict::Incomparable {
            reason: "different quality levels; a faster run at lower quality is not faster"
                .to_owned(),
        };
    }

    let p95_ratio = candidate.cpu_frame.p95_ms / baseline.cpu_frame.p95_ms.max(f64::MIN_POSITIVE);
    let p99_ratio = candidate.cpu_frame.p99_ms / baseline.cpu_frame.p99_ms.max(f64::MIN_POSITIVE);

    if candidate.stall_count > baseline.stall_count {
        return RegressionVerdict::Regressed {
            detail: format!(
                "stalls rose from {} to {}",
                baseline.stall_count, candidate.stall_count
            ),
        };
    }
    if p95_ratio > 1.0 + TOLERANCE || p99_ratio > 1.0 + TOLERANCE {
        return RegressionVerdict::Regressed {
            detail: format!(
                "p95 {:+.1}%, p99 {:+.1}%",
                (p95_ratio - 1.0) * 100.0,
                (p99_ratio - 1.0) * 100.0
            ),
        };
    }
    if p95_ratio < 1.0 - TOLERANCE {
        return RegressionVerdict::Improved {
            detail: format!("p95 {:+.1}%", (p95_ratio - 1.0) * 100.0),
        };
    }
    RegressionVerdict::Pass
}

#[cfg(test)]
mod tests {
    use super::*;
    use topovium_diagnostics::{Backend, DeviceTier};

    fn capability() -> CapabilityReport {
        CapabilityReport {
            schema_version: 1,
            topovium_version: "0.0.1".to_owned(),
            os: "linux".to_owned(),
            cpu_cores: 8,
            backend: Backend::Vulkan,
            adapter_name: "Reference Adapter".to_owned(),
            driver_info: "1.0".to_owned(),
            tier: DeviceTier::Modern,
            vram_bytes: Some(8 << 30),
            unified_memory: false,
            supports_timestamp_queries: true,
            supports_ray_query: false,
            supports_mesh_shaders: false,
            supports_multi_draw_indirect: true,
            max_texture_dimension_2d: 16384,
            applied_quirks: Vec::new(),
        }
    }

    fn result(p95: f64, p99: f64, stalls: u32) -> BenchmarkResult {
        BenchmarkResult {
            schema_version: BenchmarkResult::SCHEMA_VERSION,
            scene_id: "instances-100k".to_owned(),
            commit: "abc123".to_owned(),
            timestamp: "2026-08-20T00:00:00Z".to_owned(),
            capability: capability(),
            resolution: [1920, 1080],
            quality_levers_engaged: 0,
            cpu_frame: TimingSummary {
                p50_ms: 8.0,
                p95_ms: p95,
                p99_ms: p99,
                max_ms: p99,
                sample_count: 1000,
            },
            gpu_frame: None,
            input_to_pixel: None,
            memory_peak: MemoryPeak {
                cpu_bytes: 1 << 30,
                gpu_bytes: Some(2 << 30),
                shared_bytes: None,
            },
            stall_count: stalls,
            repro_command: "just bench instances-100k".to_owned(),
        }
    }

    #[test]
    fn a_ten_percent_p95_regression_blocks_the_merge() {
        let verdict = compare(&result(10.0, 12.0, 0), &result(11.0, 12.0, 0));
        assert!(
            matches!(verdict, RegressionVerdict::Regressed { .. }),
            "{verdict:?}"
        );
    }

    #[test]
    fn noise_within_tolerance_passes() {
        assert_eq!(
            compare(&result(10.0, 12.0, 0), &result(10.2, 12.1, 0)),
            RegressionVerdict::Pass
        );
    }

    #[test]
    fn a_p99_regression_is_caught_even_when_p95_is_unchanged() {
        // The exact case an average would hide.
        let verdict = compare(&result(10.0, 12.0, 0), &result(10.0, 20.0, 0));
        assert!(
            matches!(verdict, RegressionVerdict::Regressed { .. }),
            "{verdict:?}"
        );
    }

    #[test]
    fn a_new_stall_is_a_regression_regardless_of_percentiles() {
        let verdict = compare(&result(10.0, 12.0, 0), &result(9.0, 11.0, 1));
        assert!(
            matches!(verdict, RegressionVerdict::Regressed { .. }),
            "{verdict:?}"
        );
    }

    #[test]
    fn a_faster_run_at_lower_quality_is_not_comparable() {
        // The most common way a benchmark lies.
        let baseline = result(10.0, 12.0, 0);
        let mut candidate = result(5.0, 6.0, 0);
        candidate.quality_levers_engaged = 3;
        assert!(matches!(
            compare(&baseline, &candidate),
            RegressionVerdict::Incomparable { .. }
        ));
    }

    #[test]
    fn different_hardware_is_never_silently_compared() {
        let baseline = result(10.0, 12.0, 0);
        let mut candidate = result(5.0, 6.0, 0);
        candidate.capability.adapter_name = "Much Faster Adapter".to_owned();
        assert!(matches!(
            compare(&baseline, &candidate),
            RegressionVerdict::Incomparable { .. }
        ));
    }

    #[test]
    fn improvements_are_reported_not_just_failures() {
        let verdict = compare(&result(10.0, 12.0, 0), &result(8.0, 10.0, 0));
        assert!(
            matches!(verdict, RegressionVerdict::Improved { .. }),
            "{verdict:?}"
        );
    }

    #[test]
    fn results_round_trip_through_the_published_json() {
        let original = result(10.0, 12.0, 0);
        let parsed: BenchmarkResult = serde_json::from_str(&original.to_json().unwrap()).unwrap();
        assert_eq!(original, parsed);
    }
}
