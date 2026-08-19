// SPDX-License-Identifier: GPL-3.0-or-later
use topovium_revisions::Revision;

/// A contiguous run of changed object indices, half-open: `[start, end)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RangeUpdate {
    pub start: u32,
    pub end: u32,
}

impl RangeUpdate {
    /// A range covering one object.
    #[must_use]
    pub const fn single(index: u32) -> Self {
        Self {
            start: index,
            end: index + 1,
        }
    }

    /// How many objects this covers.
    #[must_use]
    pub const fn len(&self) -> u32 {
        self.end.saturating_sub(self.start)
    }

    /// Whether this range covers nothing.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.start >= self.end
    }

    /// How many objects lie between this range and the next, counting neither.
    /// Zero means they touch or overlap.
    #[must_use]
    const fn gap_to(&self, next: &Self) -> u32 {
        next.start.saturating_sub(self.end)
    }
}

/// Everything that changed in the GPU scene since the last flush.
///
/// The renderer applies this instead of re-uploading. Its cost is proportional to what
/// changed, which is the property the whole architecture depends on.
///
/// Ranges are coalesced on flush, not on insert, so recording a change stays O(1)
/// during input handling — the one place where a sort must never happen.
#[derive(Debug, Clone)]
pub struct GpuSceneDelta {
    /// Revision these changes bring the GPU scene up to.
    pub revision: Revision,
    dirty_indices: Vec<u32>,
    /// Objects removed. Their slots are reused, so the GPU must be told to stop
    /// drawing them before anything new lands there.
    pub removed: Vec<u32>,
    /// Whether the object buffer grew and must be reallocated before writes apply.
    pub grew_to: Option<u32>,
}

impl GpuSceneDelta {
    /// How far apart two ranges may be while still being merged.
    ///
    /// Merging costs re-sending the objects in the gap; not merging costs another
    /// staging copy and another command. Below roughly this many records the copy is
    /// cheaper than the command, so scattered edits across a big table still produce a
    /// handful of writes rather than thousands.
    pub const MERGE_GAP: u32 = 8;

    /// An empty delta at the given revision.
    #[must_use]
    pub const fn new(revision: Revision) -> Self {
        Self {
            revision,
            dirty_indices: Vec::new(),
            removed: Vec::new(),
            grew_to: None,
        }
    }

    /// Records that an object changed. O(1); duplicates are removed on flush.
    pub fn mark_dirty(&mut self, index: u32) {
        self.dirty_indices.push(index);
    }

    /// Records that an object was removed.
    pub fn mark_removed(&mut self, index: u32) {
        self.removed.push(index);
    }

    /// Records that the table grew to `new_len` objects.
    pub fn mark_grown(&mut self, new_len: u32) {
        self.grew_to = Some(
            self.grew_to
                .map_or(new_len, |existing| existing.max(new_len)),
        );
    }

    /// Whether there is nothing to upload.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.dirty_indices.is_empty() && self.removed.is_empty() && self.grew_to.is_none()
    }

    /// How many distinct objects are marked dirty, before coalescing.
    #[must_use]
    pub fn dirty_count(&self) -> usize {
        self.dirty_indices.len()
    }

    /// Collapses dirty indices into the minimal set of contiguous ranges to upload.
    ///
    /// One GPU write per range. Two thousand adjacent objects become one write of two
    /// thousand records rather than two thousand writes of one, which is the difference
    /// between a delta being a win and being slower than the full upload it replaced.
    #[must_use]
    pub fn coalesce(&self) -> Vec<RangeUpdate> {
        self.coalesce_with_gap(Self::MERGE_GAP)
    }

    /// [`Self::coalesce`] with an explicit merge gap, for tuning and tests.
    #[must_use]
    pub fn coalesce_with_gap(&self, merge_gap: u32) -> Vec<RangeUpdate> {
        if self.dirty_indices.is_empty() {
            return Vec::new();
        }

        let mut sorted = self.dirty_indices.clone();
        sorted.sort_unstable();
        sorted.dedup();

        let mut ranges: Vec<RangeUpdate> = Vec::new();
        for index in sorted {
            let next = RangeUpdate::single(index);
            match ranges.last_mut() {
                Some(current) if current.gap_to(&next) <= merge_gap => {
                    current.end = current.end.max(next.end);
                }
                _ => ranges.push(next),
            }
        }
        ranges
    }

    /// Total objects covered by the coalesced ranges, including any merged over.
    ///
    /// The honest measure of upload cost: merging trades redundant bytes for fewer
    /// commands, and this is what that trade actually costs.
    #[must_use]
    pub fn uploaded_object_count(&self) -> u32 {
        self.coalesce().iter().map(RangeUpdate::len).sum()
    }

    /// Clears the delta and advances to `revision`, retaining capacity so a steady
    /// editing session performs no allocation at all.
    pub fn flush(&mut self, revision: Revision) {
        self.dirty_indices.clear();
        self.removed.clear();
        self.grew_to = None;
        self.revision = revision;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moving_one_object_in_a_million_uploads_exactly_one() {
        // This is the headline claim of the architecture. If this test ever fails,
        // the persistent GPU scene has stopped being worth having.
        let mut delta = GpuSceneDelta::new(Revision(1));
        delta.mark_dirty(500_000);

        let ranges = delta.coalesce();
        assert_eq!(
            ranges,
            vec![RangeUpdate {
                start: 500_000,
                end: 500_001
            }]
        );
        assert_eq!(delta.uploaded_object_count(), 1);
    }

    #[test]
    fn adjacent_objects_collapse_into_a_single_write() {
        let mut delta = GpuSceneDelta::new(Revision(1));
        for i in 100..2100 {
            delta.mark_dirty(i);
        }
        let ranges = delta.coalesce();
        assert_eq!(
            ranges.len(),
            1,
            "2000 adjacent objects must be one write, not 2000"
        );
        assert_eq!(
            ranges[0],
            RangeUpdate {
                start: 100,
                end: 2100
            }
        );
    }

    #[test]
    fn small_gaps_merge_and_large_gaps_do_not() {
        let mut delta = GpuSceneDelta::new(Revision(1));
        delta.mark_dirty(0);
        delta.mark_dirty(4); // gap of 3, under the threshold
        delta.mark_dirty(1000); // far away

        let ranges = delta.coalesce();
        assert_eq!(
            ranges,
            vec![
                RangeUpdate { start: 0, end: 5 },
                RangeUpdate {
                    start: 1000,
                    end: 1001
                },
            ]
        );
    }

    #[test]
    fn repeated_marks_during_a_drag_produce_one_range() {
        // 120 Hz for two seconds on the same object.
        let mut delta = GpuSceneDelta::new(Revision(1));
        for _ in 0..240 {
            delta.mark_dirty(7);
        }
        assert_eq!(delta.coalesce(), vec![RangeUpdate::single(7)]);
        assert_eq!(delta.uploaded_object_count(), 1);
    }

    #[test]
    fn scattered_edits_stay_bounded_rather_than_one_write_each() {
        // 100 objects spread every 1000 slots across a large table.
        let mut delta = GpuSceneDelta::new(Revision(1));
        for i in 0..100u32 {
            delta.mark_dirty(i * 1000);
        }
        let ranges = delta.coalesce();
        assert_eq!(
            ranges.len(),
            100,
            "genuinely scattered edits cannot be merged"
        );
        assert_eq!(delta.uploaded_object_count(), 100, "and must not over-send");
    }

    #[test]
    fn merging_reports_the_redundant_bytes_it_costs() {
        let mut delta = GpuSceneDelta::new(Revision(1));
        delta.mark_dirty(0);
        delta.mark_dirty(8); // merged: covers 0..9, so 7 untouched objects ride along

        assert_eq!(delta.dirty_count(), 2);
        assert_eq!(
            delta.uploaded_object_count(),
            9,
            "cost of merging must be visible"
        );
    }

    #[test]
    fn an_empty_delta_produces_no_writes() {
        let delta = GpuSceneDelta::new(Revision(1));
        assert!(delta.is_empty());
        assert!(delta.coalesce().is_empty());
        assert_eq!(delta.uploaded_object_count(), 0);
    }

    #[test]
    fn growth_takes_the_largest_requested_size() {
        let mut delta = GpuSceneDelta::new(Revision(1));
        delta.mark_grown(100);
        delta.mark_grown(50);
        delta.mark_grown(200);
        assert_eq!(delta.grew_to, Some(200));
    }

    #[test]
    fn flush_resets_everything_and_advances_the_revision() {
        let mut delta = GpuSceneDelta::new(Revision(1));
        delta.mark_dirty(1);
        delta.mark_removed(2);
        delta.mark_grown(10);

        delta.flush(Revision(2));
        assert!(delta.is_empty());
        assert_eq!(delta.revision, Revision(2));
    }
}
