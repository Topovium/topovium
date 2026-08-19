// SPDX-License-Identifier: GPL-3.0-or-later
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

/// A monotonically increasing version stamp.
///
/// Revisions exist so that a job finishing late cannot overwrite newer data. Every
/// asynchronous result carries the revision of the inputs it was computed from; if the
/// current revision has moved on, the result is discarded rather than applied. Without
/// this, a slow subdivision job landing after the user's next edit silently reverts it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Revision(pub u64);

impl Revision {
    /// The revision of data that has never been modified.
    pub const ZERO: Self = Self(0);

    /// The next revision after this one.
    #[must_use]
    pub const fn next(self) -> Self {
        Self(self.0.wrapping_add(1))
    }

    /// Whether `self` is older than `other`, and therefore whether a result computed
    /// at `self` should be thrown away.
    #[must_use]
    pub const fn is_stale_against(self, other: Self) -> bool {
        self.0 < other.0
    }
}

impl fmt::Display for Revision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "r{}", self.0)
    }
}

/// A shared source of revisions.
///
/// One counter per authoring scene. Cheap to read from any thread, which matters
/// because the render thread checks staleness on every frame.
#[derive(Debug, Default)]
pub struct RevisionCounter(AtomicU64);

impl RevisionCounter {
    /// Creates a counter starting at [`Revision::ZERO`].
    #[must_use]
    pub const fn new() -> Self {
        Self(AtomicU64::new(0))
    }

    /// Reads the current revision without advancing it.
    #[must_use]
    pub fn current(&self) -> Revision {
        Revision(self.0.load(Ordering::Acquire))
    }

    /// Advances and returns the new revision.
    pub fn bump(&self) -> Revision {
        Revision(self.0.fetch_add(1, Ordering::AcqRel) + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_result_from_an_older_revision_is_stale() {
        let computed_at = Revision(4);
        let scene_now = Revision(7);
        assert!(computed_at.is_stale_against(scene_now));
        assert!(!scene_now.is_stale_against(scene_now));
    }

    #[test]
    fn counter_hands_out_increasing_revisions() {
        let counter = RevisionCounter::new();
        assert_eq!(counter.current(), Revision::ZERO);
        let a = counter.bump();
        let b = counter.bump();
        assert!(a < b);
        assert_eq!(counter.current(), b);
    }
}
