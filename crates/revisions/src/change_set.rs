// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{DirtyDomain, Revision};
use std::collections::HashMap;

/// One entity's accumulated change since the last flush.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChangedEntry {
    /// Which slot changed. Interpreted by the consumer; this crate stays agnostic.
    pub index: u32,
    /// The union of every domain touched since the last flush.
    pub domains: DirtyDomain,
}

/// What changed between two revisions.
///
/// The renderer consumes this instead of walking the scene. That is the difference
/// between a per-frame cost of `O(changed)` and `O(total objects)`, which is the
/// difference between one million instances being fine and being unusable.
///
/// Repeated edits to the same entity coalesce, so dragging an object for two seconds
/// produces one entry, not a hundred and twenty.
#[derive(Debug, Clone, Default)]
pub struct ChangeSet {
    entries: HashMap<u32, DirtyDomain>,
    revision: Revision,
}

impl ChangeSet {
    /// Creates an empty change set at the given revision.
    #[must_use]
    pub fn new(revision: Revision) -> Self {
        Self {
            entries: HashMap::new(),
            revision,
        }
    }

    /// The revision this change set describes.
    #[must_use]
    pub const fn revision(&self) -> Revision {
        self.revision
    }

    /// Records a change, merging with anything already recorded for that entity.
    pub fn mark(&mut self, index: u32, domains: DirtyDomain) {
        *self.entries.entry(index).or_insert(DirtyDomain::empty()) |= domains;
    }

    /// The union of every domain touched by any entity.
    ///
    /// Lets a consumer skip an entire subsystem in one check — if no entry touched
    /// `TOPOLOGY`, no acceleration structure needs rebuilding, and that decision costs
    /// one comparison rather than a walk.
    #[must_use]
    pub fn combined_domains(&self) -> DirtyDomain {
        self.entries
            .values()
            .fold(DirtyDomain::empty(), |acc, d| acc | *d)
    }

    /// Number of changed entities.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether nothing changed.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Iterates changed entities. Order is unspecified; consumers that need sorted
    /// ranges must sort, and are expected to, in order to coalesce GPU writes.
    pub fn iter(&self) -> impl Iterator<Item = ChangedEntry> + '_ {
        self.entries
            .iter()
            .map(|(&index, &domains)| ChangedEntry { index, domains })
    }

    /// Changed entities filtered to those touching any of `filter`.
    pub fn iter_domain(&self, filter: DirtyDomain) -> impl Iterator<Item = ChangedEntry> + '_ {
        self.iter().filter(move |e| e.domains.intersects(filter))
    }

    /// Empties the set and advances to `revision`, retaining allocated capacity so a
    /// steady-state editing session stops allocating entirely.
    pub fn flush(&mut self, revision: Revision) {
        self.entries.clear();
        self.revision = revision;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_edits_to_one_object_coalesce_into_one_entry() {
        // A two-second drag at 120 Hz must not produce 240 GPU writes.
        let mut set = ChangeSet::new(Revision(1));
        for _ in 0..240 {
            set.mark(42, DirtyDomain::TRANSFORM);
        }
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn domains_accumulate_per_entity() {
        let mut set = ChangeSet::new(Revision(1));
        set.mark(1, DirtyDomain::TRANSFORM);
        set.mark(1, DirtyDomain::MATERIAL);
        let entry = set.iter().next().unwrap();
        assert_eq!(
            entry.domains,
            DirtyDomain::TRANSFORM | DirtyDomain::MATERIAL
        );
    }

    #[test]
    fn combined_domains_lets_a_subsystem_opt_out_in_one_check() {
        let mut set = ChangeSet::new(Revision(1));
        set.mark(1, DirtyDomain::TRANSFORM);
        set.mark(2, DirtyDomain::MATERIAL);
        assert!(!set.combined_domains().intersects(DirtyDomain::TOPOLOGY));
        assert!(set.combined_domains().contains(DirtyDomain::TRANSFORM));
    }

    #[test]
    fn filtering_by_domain_yields_only_matching_entities() {
        let mut set = ChangeSet::new(Revision(1));
        set.mark(1, DirtyDomain::TRANSFORM);
        set.mark(2, DirtyDomain::MATERIAL);
        set.mark(3, DirtyDomain::TRANSFORM | DirtyDomain::GEOMETRY);

        let mut moved: Vec<u32> = set
            .iter_domain(DirtyDomain::TRANSFORM)
            .map(|e| e.index)
            .collect();
        moved.sort_unstable();
        assert_eq!(moved, vec![1, 3]);
    }

    #[test]
    fn flush_clears_entries_and_advances_the_revision() {
        let mut set = ChangeSet::new(Revision(1));
        set.mark(1, DirtyDomain::TRANSFORM);
        set.flush(Revision(2));
        assert!(set.is_empty());
        assert_eq!(set.revision(), Revision(2));
    }
}
