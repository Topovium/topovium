// SPDX-License-Identifier: GPL-3.0-or-later
//! Priority-scheduled background work.
//!
//! Three properties matter more than throughput here:
//!
//! 1. **Priority.** Work the user is waiting on right now outranks everything.
//! 2. **Cancellation.** A long job whose result is already obsolete must stop, not
//!    finish politely and waste a core.
//! 3. **Revision tagging.** Every result carries the revision of its inputs, so a
//!    slow job landing after the user's next edit is discarded instead of reverting it.
//!
//! Async is not a substitute for this. `async` schedules *waiting*; this schedules
//! *CPU work*, and conflating the two is how a modifier evaluation ends up blocking a
//! frame. See `AGENTS.md` §4.3.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

mod cancel;
mod pool;
mod priority;

pub use cancel::{CancellationToken, Cancelled};
pub use pool::{JobPool, JobPoolConfig};
pub use priority::Priority;

use topovium_revisions::Revision;

/// A completed job's output plus the revision it was computed from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Versioned<T> {
    pub value: T,
    pub computed_at: Revision,
}

impl<T> Versioned<T> {
    /// Tags a value with the revision it was computed from.
    pub const fn new(value: T, computed_at: Revision) -> Self {
        Self { value, computed_at }
    }

    /// Returns the value only if it is still current.
    ///
    /// The whole reason this type exists: a result older than the scene must never be
    /// applied, because applying it silently undoes the user's most recent edit.
    pub fn accept_if_current(self, scene_revision: Revision) -> Option<T> {
        if self.computed_at.is_stale_against(scene_revision) {
            None
        } else {
            Some(self.value)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_late_result_from_an_old_revision_is_rejected() {
        let result = Versioned::new("subdivided mesh", Revision(3));
        assert_eq!(result.accept_if_current(Revision(5)), None);
    }

    #[test]
    fn a_current_result_is_accepted() {
        let result = Versioned::new("subdivided mesh", Revision(5));
        assert_eq!(
            result.accept_if_current(Revision(5)),
            Some("subdivided mesh")
        );
    }
}
