// SPDX-License-Identifier: GPL-3.0-or-later
use serde::{Deserialize, Serialize};

/// Which graphics API a device is being driven through.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Backend {
    Vulkan,
    Metal,
    Dx12,
    /// WebGPU in a browser.
    WebGpu,
    /// Software rasterisation. Present so a machine without a working driver still
    /// runs rather than showing a black window.
    Cpu,
}

/// A device's capability class.
///
/// Tiers exist so quality decisions are made once, from measured capability, instead
/// of scattered across the codebase as vendor-string checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DeviceTier {
    /// Basic raster only. Low mips, limited shadows, no local path tracing. Editing
    /// works fully; heavy rendering is offloaded.
    Basic,
    /// Full PBR, GPU culling, dynamic resolution, texture streaming.
    Modern,
    /// Higher LODs, better shadows, ray queries where stable, GPU denoising.
    HighEnd,
}

/// What a machine can actually do, captured at startup.
///
/// Emitted by `topovium capability-dump` and attached to every benchmark result. A
/// performance number without this is unreproducible, and an unreproducible number is
/// not evidence — see `AGENTS.md` §5.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityReport {
    /// Schema version, so old reports stay readable as fields are added.
    pub schema_version: u32,
    pub topovium_version: String,
    pub os: String,
    pub cpu_cores: usize,
    pub backend: Backend,
    pub adapter_name: String,
    pub driver_info: String,
    pub tier: DeviceTier,
    /// Dedicated video memory, where the platform reports it. `None` on unified
    /// memory devices, where a separate figure would be meaningless.
    pub vram_bytes: Option<u64>,
    /// Whether CPU and GPU share one memory pool. Determines whether RAM and VRAM
    /// budgets may be counted separately, and double-counting here is how mobile
    /// builds get killed by the OS.
    pub unified_memory: bool,
    pub supports_timestamp_queries: bool,
    pub supports_ray_query: bool,
    pub supports_mesh_shaders: bool,
    pub supports_multi_draw_indirect: bool,
    pub max_texture_dimension_2d: u32,
    /// Known driver problems matched against this adapter, with the workaround applied.
    pub applied_quirks: Vec<String>,
}

impl CapabilityReport {
    /// Current schema version.
    pub const SCHEMA_VERSION: u32 = 1;

    /// Serialises to pretty JSON for `topovium capability-dump` and bug reports.
    ///
    /// # Errors
    /// Returns an error if serialisation fails.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Texture dimension above the WebGPU baseline of 8192, which in practice
    /// separates desktop- and flagship-mobile-class GPUs from entry-level ones.
    const DESKTOP_CLASS_TEXTURE_DIMENSION: u32 = 16384;

    /// Classifies a device from its measured features.
    ///
    /// Feature-based, never a vendor or model string: a name-based allowlist is wrong
    /// the day a new device ships, and there are thousands of Android GPUs.
    ///
    /// Note that `supports_multi_draw_indirect` is treated as sufficient evidence of a
    /// capable device but never as necessary. Metal reaches the same result through
    /// indirect command buffers and does not report that feature at all, so requiring
    /// it would classify an Apple M-series GPU as entry-level — which an earlier
    /// version of this function did, on real hardware.
    #[must_use]
    pub fn classify(
        supports_ray_query: bool,
        supports_multi_draw_indirect: bool,
        supports_timestamp_queries: bool,
        max_texture_dimension_2d: u32,
    ) -> DeviceTier {
        let desktop_class = max_texture_dimension_2d >= Self::DESKTOP_CLASS_TEXTURE_DIMENSION;

        // Two independent routes to Modern, because no single feature identifies a
        // capable GPU across all four backends.
        let modern = (desktop_class && supports_timestamp_queries)
            || (supports_multi_draw_indirect && max_texture_dimension_2d >= 8192);

        if supports_ray_query && desktop_class {
            DeviceTier::HighEnd
        } else if modern {
            DeviceTier::Modern
        } else {
            DeviceTier::Basic
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report() -> CapabilityReport {
        CapabilityReport {
            schema_version: CapabilityReport::SCHEMA_VERSION,
            topovium_version: "0.0.1".to_owned(),
            os: "macos".to_owned(),
            cpu_cores: 10,
            backend: Backend::Metal,
            adapter_name: "Test Adapter".to_owned(),
            driver_info: "test".to_owned(),
            tier: DeviceTier::HighEnd,
            vram_bytes: None,
            unified_memory: true,
            supports_timestamp_queries: true,
            supports_ray_query: true,
            supports_mesh_shaders: true,
            supports_multi_draw_indirect: true,
            max_texture_dimension_2d: 16384,
            applied_quirks: Vec::new(),
        }
    }

    #[test]
    fn report_round_trips_through_json() {
        let original = report();
        let json = original.to_json().unwrap();
        let parsed: CapabilityReport = serde_json::from_str(&json).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn tier_is_derived_from_features_not_device_names() {
        assert_eq!(
            CapabilityReport::classify(true, true, true, 16384),
            DeviceTier::HighEnd
        );
        assert_eq!(
            CapabilityReport::classify(false, true, true, 8192),
            DeviceTier::Modern
        );
        assert_eq!(
            CapabilityReport::classify(false, false, false, 4096),
            DeviceTier::Basic
        );
    }

    #[test]
    fn a_metal_gpu_without_multi_draw_indirect_is_not_called_entry_level() {
        // Regression: an Apple M4 Pro was classified Basic because Metal reports no
        // MULTI_DRAW_INDIRECT_COUNT, using indirect command buffers instead. Found by
        // running `topovium capability-dump` on real hardware.
        assert_eq!(
            CapabilityReport::classify(false, false, true, 16384),
            DeviceTier::Modern
        );
    }

    #[test]
    fn a_genuinely_weak_device_is_still_classified_basic() {
        assert_eq!(
            CapabilityReport::classify(false, false, true, 4096),
            DeviceTier::Basic
        );
        assert_eq!(
            CapabilityReport::classify(false, false, false, 8192),
            DeviceTier::Basic
        );
    }

    #[test]
    fn tiers_are_ordered_so_quality_gates_can_compare_them() {
        assert!(DeviceTier::Basic < DeviceTier::Modern);
        assert!(DeviceTier::Modern < DeviceTier::HighEnd);
    }

    #[test]
    fn unified_memory_devices_report_no_separate_vram() {
        // Counting shared memory twice is how a mobile build gets OOM-killed.
        let r = report();
        assert!(r.unified_memory);
        assert!(r.vram_bytes.is_none());
    }
}
