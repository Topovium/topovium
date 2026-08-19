// SPDX-License-Identifier: GPL-3.0-or-later
use bitflags::bitflags;

bitflags! {
    /// What kind of change happened.
    ///
    /// A single "dirty" bit is the difference between a responsive editor and a slow
    /// one. Dragging a light must not rebuild meshlets; changing roughness must not
    /// invalidate geometry; orbiting the camera must not run the dependency graph at
    /// all. Typed domains make each of those a compile-time-visible decision instead
    /// of a comment nobody reads.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct DirtyDomain: u32 {
        /// Object moved, rotated, or scaled. Cheapest possible change: a small GPU
        /// range write, and for ray tracing a TLAS refit.
        const TRANSFORM   = 1 << 0;
        /// Object shown or hidden.
        const VISIBILITY  = 1 << 1;
        /// Material assignment or parameter changed.
        const MATERIAL    = 1 << 2;
        /// Vertex attributes changed, positions unchanged.
        const ATTRIBUTES  = 1 << 3;
        /// Vertex positions changed, topology unchanged. Allows a BLAS refit.
        const GEOMETRY    = 1 << 4;
        /// Element count or connectivity changed. Forces a BLAS rebuild.
        const TOPOLOGY    = 1 << 5;
        /// A light's parameters or transform changed.
        const LIGHTING    = 1 << 6;
        /// Animation evaluation produced new values.
        const ANIMATION   = 1 << 7;
        /// Acceleration structures need attention.
        const BVH         = 1 << 8;
        /// Affects rendering only; the authoring scene is untouched and the file is
        /// not dirty. Viewport display options live here.
        const RENDER_ONLY = 1 << 9;
    }
}

impl DirtyDomain {
    /// Domains that force acceleration structures to be rebuilt rather than refitted.
    pub const REQUIRES_BVH_REBUILD: Self = Self::TOPOLOGY;

    /// Domains that a refit can absorb.
    pub const REQUIRES_BVH_REFIT: Self = Self::TRANSFORM.union(Self::GEOMETRY);

    /// Domains that change what is written to the project file.
    ///
    /// [`Self::RENDER_ONLY`] is deliberately excluded: toggling wireframe must not
    /// mark a user's project as having unsaved changes.
    pub const PERSISTENT: Self = Self::all().difference(Self::RENDER_ONLY);

    /// Whether this change requires re-running the dependency graph.
    ///
    /// Transform, visibility, and render-only changes do not: they are applied
    /// directly to evaluated output.
    #[must_use]
    pub const fn needs_reevaluation(self) -> bool {
        self.intersects(
            Self::ATTRIBUTES
                .union(Self::GEOMETRY)
                .union(Self::TOPOLOGY)
                .union(Self::ANIMATION),
        )
    }

    /// Whether this change makes the project file dirty.
    #[must_use]
    pub const fn marks_project_dirty(self) -> bool {
        self.intersects(Self::PERSISTENT)
    }

    /// Whether path-tracer temporal history must be discarded.
    ///
    /// Deliberately narrow. Discarding history on every trivial change is why
    /// interactive preview feels slow in other renderers.
    #[must_use]
    pub const fn invalidates_temporal_history(self) -> bool {
        self.intersects(
            Self::TRANSFORM
                .union(Self::GEOMETRY)
                .union(Self::TOPOLOGY)
                .union(Self::MATERIAL)
                .union(Self::LIGHTING)
                .union(Self::VISIBILITY),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moving_a_light_does_not_trigger_reevaluation() {
        let change = DirtyDomain::TRANSFORM | DirtyDomain::LIGHTING;
        assert!(!change.needs_reevaluation());
    }

    #[test]
    fn changing_roughness_does_not_touch_geometry_or_the_graph() {
        let change = DirtyDomain::MATERIAL;
        assert!(!change.needs_reevaluation());
        assert!(!change.intersects(DirtyDomain::GEOMETRY | DirtyDomain::TOPOLOGY));
    }

    #[test]
    fn viewport_display_options_do_not_dirty_the_project() {
        assert!(!DirtyDomain::RENDER_ONLY.marks_project_dirty());
        assert!(DirtyDomain::TRANSFORM.marks_project_dirty());
    }

    #[test]
    fn topology_forces_rebuild_while_deformation_only_refits() {
        assert!(DirtyDomain::TOPOLOGY.intersects(DirtyDomain::REQUIRES_BVH_REBUILD));
        assert!(!DirtyDomain::GEOMETRY.intersects(DirtyDomain::REQUIRES_BVH_REBUILD));
        assert!(DirtyDomain::GEOMETRY.intersects(DirtyDomain::REQUIRES_BVH_REFIT));
    }
}
