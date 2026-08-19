// SPDX-License-Identifier: GPL-3.0-or-later
use bytemuck::{Pod, Zeroable};
use topovium_foundation::{Transform, math::Vec4};

/// Per-object state the GPU tests without CPU involvement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ObjectFlags(pub u32);

impl ObjectFlags {
    pub const VISIBLE: Self = Self(1 << 0);
    pub const CASTS_SHADOW: Self = Self(1 << 1);
    pub const SELECTED: Self = Self(1 << 2);
    pub const TRANSPARENT: Self = Self(1 << 3);
    /// Excluded from occlusion culling. For objects whose bounds are unreliable, such
    /// as ones displaced in a vertex shader.
    pub const SKIP_OCCLUSION: Self = Self(1 << 4);

    /// Union of two flag sets.
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Whether every bit in `other` is set.
    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

/// One object, laid out exactly as the GPU reads it.
///
/// `repr(C)` and 16-byte aligned so it maps to a storage buffer with no packing
/// surprises, and so a `RangeUpdate` can be a byte copy rather than a per-field
/// conversion. The layout is asserted by a test, because a silent change here
/// corrupts every object in the scene rather than failing loudly.
#[repr(C, align(16))]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct ObjectRecord {
    /// Object-to-world, row-major, 4x4.
    pub transform: [f32; 16],
    /// Previous frame's transform, for motion vectors and temporal reprojection.
    pub previous_transform: [f32; 16],
    /// World-space bounding sphere: xyz centre, w radius. Read by the culling compute
    /// shader; a sphere is one dot product per plane, an AABB is eight.
    pub bounds: [f32; 4],
    pub mesh_handle: u32,
    pub material_handle: u32,
    pub flags: u32,
    /// Explicit tail padding to a 16-byte boundary. Named rather than implicit so that
    /// adding a field forces a deliberate decision about layout.
    pub _padding: u32,
}

impl ObjectRecord {
    /// Size in bytes. Used to turn object indices into buffer offsets.
    pub const SIZE: usize = std::mem::size_of::<Self>();

    /// Builds a record from editor-side values.
    #[must_use]
    pub fn new(
        transform: Transform,
        bounds: Vec4,
        mesh_handle: u32,
        material_handle: u32,
        flags: ObjectFlags,
    ) -> Self {
        let matrix = transform.to_matrix().to_cols_array();
        Self {
            transform: matrix,
            previous_transform: matrix,
            bounds: bounds.to_array(),
            mesh_handle,
            material_handle,
            flags: flags.0,
            _padding: 0,
        }
    }

    /// Updates the transform, retaining the old one for motion vectors.
    ///
    /// Keeping the previous transform here rather than in a parallel buffer means one
    /// contiguous write covers both, which halves the number of ranges a drag produces.
    pub fn set_transform(&mut self, transform: Transform) {
        self.previous_transform = self.transform;
        self.transform = transform.to_matrix().to_cols_array();
    }
}

impl Default for ObjectRecord {
    fn default() -> Self {
        Self::zeroed()
    }
}

/// The CPU-side mirror of the GPU object buffer.
///
/// Columnar and contiguous, so a run of changed objects becomes one memcpy. Deliberately
/// **not** `Vec<Arc<Mutex<Object>>>`: a million objects would mean a million allocations
/// and a million locks, and pointer chasing would defeat every cache line. See
/// `AGENTS.md` §4.3.
#[derive(Debug, Default)]
pub struct ObjectTable {
    records: Vec<ObjectRecord>,
}

impl ObjectTable {
    /// An empty table.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    /// A table preallocated for `capacity` objects.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            records: Vec::with_capacity(capacity),
        }
    }

    /// Number of objects.
    #[must_use]
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Whether the table is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Appends an object and returns its index.
    pub fn push(&mut self, record: ObjectRecord) -> u32 {
        let index = u32::try_from(self.records.len()).unwrap_or(u32::MAX);
        self.records.push(record);
        index
    }

    /// Borrows an object.
    #[must_use]
    pub fn get(&self, index: u32) -> Option<&ObjectRecord> {
        self.records.get(index as usize)
    }

    /// Mutably borrows an object. The caller records the change in a
    /// [`crate::GpuSceneDelta`]; this type does not track dirtiness itself, because
    /// change tracking belongs to the transaction that made the change.
    pub fn get_mut(&mut self, index: u32) -> Option<&mut ObjectRecord> {
        self.records.get_mut(index as usize)
    }

    /// The whole table as bytes, for the initial upload only.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.records)
    }

    /// A half-open range of records as bytes, for a delta upload.
    #[must_use]
    pub fn range_bytes(&self, start: u32, end: u32) -> Option<&[u8]> {
        let slice = self.records.get(start as usize..end as usize)?;
        Some(bytemuck::cast_slice(slice))
    }

    /// Byte offset of an object within the GPU buffer.
    #[must_use]
    pub const fn byte_offset(index: u32) -> u64 {
        index as u64 * ObjectRecord::SIZE as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use topovium_foundation::math::Vec3;

    #[test]
    fn record_layout_is_stable_and_gpu_compatible() {
        // A silent layout change here corrupts every object in the scene, so it is
        // pinned by a test rather than left to chance.
        assert_eq!(ObjectRecord::SIZE, 160);
        assert_eq!(
            ObjectRecord::SIZE % 16,
            0,
            "must stay 16-byte aligned for std430"
        );
        assert_eq!(std::mem::align_of::<ObjectRecord>(), 16);
    }

    #[test]
    fn setting_a_transform_preserves_the_previous_one_for_motion_vectors() {
        let mut record = ObjectRecord::new(
            Transform::IDENTITY,
            Vec4::new(0.0, 0.0, 0.0, 1.0),
            0,
            0,
            ObjectFlags::VISIBLE,
        );
        let original = record.transform;
        record.set_transform(Transform::from_translation(Vec3::new(5.0, 0.0, 0.0)));

        assert_eq!(record.previous_transform, original);
        assert_ne!(record.transform, original);
    }

    #[test]
    fn byte_offsets_address_the_right_record() {
        assert_eq!(ObjectTable::byte_offset(0), 0);
        assert_eq!(ObjectTable::byte_offset(1), ObjectRecord::SIZE as u64);
        assert_eq!(
            ObjectTable::byte_offset(1000),
            1000 * ObjectRecord::SIZE as u64
        );
    }

    #[test]
    fn a_range_yields_exactly_the_bytes_of_the_records_it_covers() {
        let mut table = ObjectTable::new();
        for _ in 0..10 {
            table.push(ObjectRecord::default());
        }
        let bytes = table.range_bytes(2, 5).unwrap();
        assert_eq!(bytes.len(), 3 * ObjectRecord::SIZE);
        assert!(
            table.range_bytes(5, 100).is_none(),
            "out of range must not panic"
        );
    }

    #[test]
    fn flags_compose_and_test_correctly() {
        let flags = ObjectFlags::VISIBLE.union(ObjectFlags::CASTS_SHADOW);
        assert!(flags.contains(ObjectFlags::VISIBLE));
        assert!(flags.contains(ObjectFlags::CASTS_SHADOW));
        assert!(!flags.contains(ObjectFlags::SELECTED));
    }
}
