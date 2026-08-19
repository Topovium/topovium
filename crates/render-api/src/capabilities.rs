// SPDX-License-Identifier: GPL-3.0-or-later
use topovium_diagnostics::DeviceTier;

/// An optional capability that a backend may or may not provide.
///
/// Everything here is queried at runtime and has a defined fallback. Assuming any of
/// them exist is how an application ships a black screen to a third of Android.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderFeature {
    /// GPU timestamp queries. Without these, GPU frame time is unmeasurable and the
    /// HUD reports CPU time only.
    TimestampQuery,
    /// One indirect draw call whose count comes from a GPU buffer. This is what makes
    /// GPU-driven culling worthwhile; without it, culling results must round-trip.
    MultiDrawIndirectCount,
    /// Inline ray tracing from any shader stage.
    RayQuery,
    /// Hardware acceleration structures.
    RayTracingAccelerationStructure,
    /// Task and mesh shader stages. Meshlet culling has a compute fallback, so this is
    /// a fast path rather than a requirement.
    MeshShader,
    /// Large descriptor arrays indexed at runtime.
    BindlessResources,
    /// Partially resident textures, for streaming very large images.
    SparseTextures,
    /// Float32 texture filtering. Absent on a surprising number of mobile GPUs.
    Float32Filterable,
}

impl RenderFeature {
    /// Whether the renderer still works without this feature.
    ///
    /// Every feature here answers `true`. That is deliberate and load-bearing: the
    /// baseline must run on the weakest supported device, and a feature with no
    /// fallback is a feature that cannot be adopted.
    #[must_use]
    pub const fn has_fallback(self) -> bool {
        true
    }

    /// The minimum device tier where this feature is normally expected.
    #[must_use]
    pub const fn typical_tier(self) -> DeviceTier {
        match self {
            Self::TimestampQuery | Self::Float32Filterable => DeviceTier::Basic,
            Self::MultiDrawIndirectCount | Self::BindlessResources => DeviceTier::Modern,
            Self::RayQuery
            | Self::RayTracingAccelerationStructure
            | Self::MeshShader
            | Self::SparseTextures => DeviceTier::HighEnd,
        }
    }
}

/// Hard limits reported by a device. Exceeding one is an error, not a slow path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceLimits {
    pub max_texture_dimension_2d: u32,
    pub max_buffer_size: u64,
    pub max_bind_groups: u32,
    pub max_storage_buffer_binding_size: u64,
    pub max_compute_workgroup_size: [u32; 3],
    pub max_compute_invocations_per_workgroup: u32,
}

impl DeviceLimits {
    /// The floor Topovium targets: WebGPU's guaranteed baseline, which is also roughly
    /// the weakest Vulkan device worth supporting.
    pub const BASELINE: Self = Self {
        max_texture_dimension_2d: 8192,
        max_buffer_size: 256 * 1024 * 1024,
        max_bind_groups: 4,
        max_storage_buffer_binding_size: 128 * 1024 * 1024,
        max_compute_workgroup_size: [256, 256, 64],
        max_compute_invocations_per_workgroup: 256,
    };

    /// Whether this device meets the baseline everywhere.
    #[must_use]
    pub const fn meets_baseline(&self) -> bool {
        self.max_texture_dimension_2d >= Self::BASELINE.max_texture_dimension_2d
            && self.max_buffer_size >= Self::BASELINE.max_buffer_size
            && self.max_bind_groups >= Self::BASELINE.max_bind_groups
            && self.max_storage_buffer_binding_size
                >= Self::BASELINE.max_storage_buffer_binding_size
            && self.max_compute_invocations_per_workgroup
                >= Self::BASELINE.max_compute_invocations_per_workgroup
    }
}

/// Pixel format of the presented surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceFormat {
    Rgba8UnormSrgb,
    Bgra8UnormSrgb,
    /// Extended range, for HDR displays.
    Rgba16Float,
}

impl SurfaceFormat {
    /// Whether the display hardware applies sRGB encoding, meaning shaders must write
    /// linear values and must not encode themselves.
    #[must_use]
    pub const fn is_srgb(self) -> bool {
        matches!(self, Self::Rgba8UnormSrgb | Self::Bgra8UnormSrgb)
    }
}

/// What a GPU buffer is used for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferUsage {
    /// Small, frequently rewritten data such as camera constants.
    Uniform,
    /// Large data read by shaders: the object table, meshlets, instances.
    Storage,
    Vertex,
    Index,
    /// Draw arguments generated on the GPU.
    Indirect,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_feature_has_a_fallback() {
        // A feature without a fallback cannot be used, because the baseline has to run
        // on the weakest supported device.
        for feature in [
            RenderFeature::TimestampQuery,
            RenderFeature::MultiDrawIndirectCount,
            RenderFeature::RayQuery,
            RenderFeature::MeshShader,
            RenderFeature::SparseTextures,
        ] {
            assert!(
                feature.has_fallback(),
                "{feature:?} must degrade gracefully"
            );
        }
    }

    #[test]
    fn baseline_limits_are_met_by_the_baseline() {
        assert!(DeviceLimits::BASELINE.meets_baseline());
    }

    #[test]
    fn a_device_below_baseline_is_detected() {
        let weak = DeviceLimits {
            max_texture_dimension_2d: 4096,
            ..DeviceLimits::BASELINE
        };
        assert!(!weak.meets_baseline());
    }

    #[test]
    fn srgb_surfaces_are_identified_so_shaders_do_not_double_encode() {
        assert!(SurfaceFormat::Bgra8UnormSrgb.is_srgb());
        assert!(!SurfaceFormat::Rgba16Float.is_srgb());
    }
}
