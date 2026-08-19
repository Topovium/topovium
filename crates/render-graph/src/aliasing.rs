// SPDX-License-Identifier: GPL-3.0-or-later
use crate::graph::ResourceId;

/// The span of passes over which a transient resource holds live data.
///
/// Half-open in pass-order indices: `[first_write, last_read)`. Outside it, the memory
/// is free for something else.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceLifetime {
    pub resource: ResourceId,
    pub first_use: usize,
    pub last_use: usize,
    pub size_bytes: u64,
}

impl ResourceLifetime {
    /// Whether two resources are live at the same time and therefore need separate memory.
    #[must_use]
    pub const fn overlaps(&self, other: &Self) -> bool {
        self.first_use <= other.last_use && other.first_use <= self.last_use
    }
}

/// Which resources share which memory blocks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AliasPlan {
    /// One entry per block: the resources assigned to it, in assignment order.
    pub blocks: Vec<Vec<ResourceId>>,
    /// Bytes actually allocated: the sum of each block's largest member.
    pub total_bytes: u64,
    /// Bytes that would have been needed without aliasing.
    pub unaliased_bytes: u64,
}

impl AliasPlan {
    /// Fraction of memory saved, 0.0 to 1.0.
    #[must_use]
    pub fn savings_ratio(&self) -> f64 {
        if self.unaliased_bytes == 0 {
            return 0.0;
        }
        1.0 - (self.total_bytes as f64 / self.unaliased_bytes as f64)
    }
}

/// Packs transient resources into the fewest memory blocks their lifetimes permit.
///
/// Greedy first-fit over resources sorted largest-first. Optimal packing is NP-hard and
/// this runs once per graph change, so a good answer now beats a perfect answer later —
/// especially since the input is tens of resources, not thousands.
///
/// Sorting largest-first matters: placing the big targets before the small ones lets
/// small resources fill the gaps, which first-fit in arbitrary order does not achieve.
#[must_use]
pub fn plan_aliasing(lifetimes: &[ResourceLifetime]) -> AliasPlan {
    let unaliased_bytes: u64 = lifetimes.iter().map(|l| l.size_bytes).sum();

    let mut sorted: Vec<&ResourceLifetime> = lifetimes.iter().collect();
    sorted.sort_by(|a, b| {
        b.size_bytes
            .cmp(&a.size_bytes)
            .then_with(|| a.resource.0.cmp(&b.resource.0))
    });

    let mut blocks: Vec<Vec<&ResourceLifetime>> = Vec::new();
    for lifetime in sorted {
        let slot = blocks
            .iter_mut()
            .find(|block| block.iter().all(|existing| !existing.overlaps(lifetime)));
        match slot {
            Some(block) => block.push(lifetime),
            None => blocks.push(vec![lifetime]),
        }
    }

    let total_bytes = blocks
        .iter()
        .map(|block| block.iter().map(|l| l.size_bytes).max().unwrap_or(0))
        .sum();

    AliasPlan {
        blocks: blocks
            .iter()
            .map(|b| b.iter().map(|l| l.resource).collect())
            .collect(),
        total_bytes,
        unaliased_bytes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lifetime(id: u32, first: usize, last: usize, mb: u64) -> ResourceLifetime {
        ResourceLifetime {
            resource: ResourceId(id),
            first_use: first,
            last_use: last,
            size_bytes: mb * 1024 * 1024,
        }
    }

    #[test]
    fn resources_with_disjoint_lifetimes_share_one_block() {
        // AO, bloom, and the denoiser temporary never coexist. On a tablet this is the
        // difference between 24 MB and 8 MB of transient memory.
        let plan = plan_aliasing(&[
            lifetime(0, 0, 2, 8),
            lifetime(1, 3, 5, 8),
            lifetime(2, 6, 8, 8),
        ]);
        assert_eq!(plan.blocks.len(), 1);
        assert_eq!(plan.total_bytes, 8 * 1024 * 1024);
        assert_eq!(plan.unaliased_bytes, 24 * 1024 * 1024);
    }

    #[test]
    fn overlapping_resources_never_share_memory() {
        // Getting this wrong corrupts a live render target, so it is asserted directly.
        let plan = plan_aliasing(&[lifetime(0, 0, 5, 8), lifetime(1, 3, 8, 8)]);
        assert_eq!(plan.blocks.len(), 2);
        assert_eq!(plan.total_bytes, 16 * 1024 * 1024);
    }

    #[test]
    fn a_block_is_sized_by_its_largest_member() {
        let plan = plan_aliasing(&[lifetime(0, 0, 1, 16), lifetime(1, 2, 3, 4)]);
        assert_eq!(plan.blocks.len(), 1);
        assert_eq!(plan.total_bytes, 16 * 1024 * 1024);
    }

    #[test]
    fn touching_lifetimes_are_treated_as_overlapping() {
        // A resource read at pass 3 and one first written at pass 3 are both live
        // during that pass. Treating them as disjoint would corrupt the read.
        let a = lifetime(0, 0, 3, 8);
        let b = lifetime(1, 3, 6, 8);
        assert!(a.overlaps(&b));
        assert_eq!(plan_aliasing(&[a, b]).blocks.len(), 2);
    }

    #[test]
    fn savings_are_reported_for_the_memory_hud() {
        let plan = plan_aliasing(&[lifetime(0, 0, 1, 8), lifetime(1, 2, 3, 8)]);
        assert!((plan.savings_ratio() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn no_resources_means_no_allocation_and_no_division_by_zero() {
        let plan = plan_aliasing(&[]);
        assert_eq!(plan.total_bytes, 0);
        assert!((plan.savings_ratio() - 0.0).abs() < f64::EPSILON);
    }
}
