// SPDX-License-Identifier: GPL-3.0-or-later
use topovium_diagnostics::{Backend, CapabilityReport};
use topovium_render_api::{DeviceLimits, RenderFeature};

/// Translates `wgpu`'s limits into the backend-neutral form.
#[must_use]
pub fn map_limits(limits: &wgpu::Limits) -> DeviceLimits {
    DeviceLimits {
        max_texture_dimension_2d: limits.max_texture_dimension_2d,
        max_buffer_size: limits.max_buffer_size,
        max_bind_groups: limits.max_bind_groups,
        max_storage_buffer_binding_size: limits.max_storage_buffer_binding_size,
        max_compute_workgroup_size: [
            limits.max_compute_workgroup_size_x,
            limits.max_compute_workgroup_size_y,
            limits.max_compute_workgroup_size_z,
        ],
        max_compute_invocations_per_workgroup: limits.max_compute_invocations_per_workgroup,
    }
}

/// Whether a `wgpu` feature set provides one of our optional capabilities.
///
/// Features Topovium does not yet map are reported absent rather than assumed present.
/// Absent means "use the fallback", which always works; wrongly assuming present means
/// a black screen on someone's laptop.
#[must_use]
pub fn map_features(features: wgpu::Features, feature: RenderFeature) -> bool {
    match feature {
        RenderFeature::TimestampQuery => features.contains(wgpu::Features::TIMESTAMP_QUERY),
        RenderFeature::MultiDrawIndirectCount => {
            features.contains(wgpu::Features::MULTI_DRAW_INDIRECT_COUNT)
        }
        RenderFeature::Float32Filterable => features.contains(wgpu::Features::FLOAT32_FILTERABLE),
        RenderFeature::BindlessResources => features.contains(
            wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING,
        ),
        // Ray tracing, mesh shaders, and sparse residency are still experimental in
        // wgpu and vary by backend. They are reached through a native fast path, not
        // through this baseline, so the baseline reports them absent and uses the
        // compute fallback. Revisit when the upstream API stabilises.
        RenderFeature::RayQuery
        | RenderFeature::RayTracingAccelerationStructure
        | RenderFeature::MeshShader
        | RenderFeature::SparseTextures => false,
    }
}

/// Builds a capability report from an adapter.
///
/// Attached to every benchmark result and every bug report. A performance number
/// without this is not reproducible, and an irreproducible number is not evidence.
#[must_use]
pub fn describe_adapter(
    adapter_info: &wgpu::AdapterInfo,
    features: wgpu::Features,
    limits: &wgpu::Limits,
    topovium_version: &str,
) -> CapabilityReport {
    let backend = match adapter_info.backend {
        wgpu::Backend::Vulkan => Backend::Vulkan,
        wgpu::Backend::Metal => Backend::Metal,
        wgpu::Backend::Dx12 => Backend::Dx12,
        wgpu::Backend::BrowserWebGpu => Backend::WebGpu,
        _ => Backend::Cpu,
    };

    let supports_ray_query = map_features(features, RenderFeature::RayQuery);
    let supports_multi_draw_indirect =
        map_features(features, RenderFeature::MultiDrawIndirectCount);
    let supports_timestamp_queries = map_features(features, RenderFeature::TimestampQuery);
    let tier = CapabilityReport::classify(
        supports_ray_query,
        supports_multi_draw_indirect,
        supports_timestamp_queries,
        limits.max_texture_dimension_2d,
    );

    // Integrated and software adapters share memory with the CPU. Reporting a separate
    // VRAM figure for them would double-count, which on mobile is the difference
    // between a correct budget and being killed by the OS.
    let unified_memory = !matches!(adapter_info.device_type, wgpu::DeviceType::DiscreteGpu);

    CapabilityReport {
        schema_version: CapabilityReport::SCHEMA_VERSION,
        topovium_version: topovium_version.to_owned(),
        os: std::env::consts::OS.to_owned(),
        cpu_cores: std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get),
        backend,
        adapter_name: adapter_info.name.clone(),
        driver_info: format!("{} {}", adapter_info.driver, adapter_info.driver_info)
            .trim()
            .to_owned(),
        tier,
        vram_bytes: None,
        unified_memory,
        supports_timestamp_queries,
        supports_ray_query,
        supports_mesh_shaders: map_features(features, RenderFeature::MeshShader),
        supports_multi_draw_indirect,
        max_texture_dimension_2d: limits.max_texture_dimension_2d,
        applied_quirks: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn downlevel_defaults_map_to_our_baseline_shape() {
        let limits = map_limits(&wgpu::Limits::downlevel_defaults());
        assert!(limits.max_texture_dimension_2d > 0);
        assert!(limits.max_buffer_size > 0);
    }

    #[test]
    fn experimental_features_are_reported_absent_rather_than_assumed() {
        // Assuming a capability that is missing ships a black screen; assuming it is
        // missing when present only costs the fast path.
        let none = wgpu::Features::empty();
        assert!(!map_features(none, RenderFeature::RayQuery));
        assert!(!map_features(none, RenderFeature::MeshShader));
        assert!(!map_features(none, RenderFeature::TimestampQuery));
    }

    #[test]
    fn a_device_without_indirect_count_is_not_classified_high_end() {
        assert_eq!(
            CapabilityReport::classify(false, false, false, 8192),
            topovium_diagnostics::DeviceTier::Basic
        );
    }
}
