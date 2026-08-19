// SPDX-License-Identifier: GPL-3.0-or-later
/// How urgent a job is.
///
/// Ordered so that `Priority::InputCritical < Priority::Background`, letting a
/// scheduler sort ascending and take the front. The numbering matches the design
/// document's P0–P4.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Priority {
    /// P0 — the user is manipulating something and this result is needed to draw it.
    /// Nothing preempts this.
    InputCritical = 0,
    /// P1 — needed for the next frame's visible output.
    VisibleFrame = 1,
    /// P2 — likely to become visible shortly. Predictive streaming lives here.
    NearFuture = 2,
    /// P3 — LODs, thumbnails, cache compression. Yields to everything above.
    Background = 3,
    /// P4 — compaction, cleanup, statistics. Runs when nothing else wants a core.
    Maintenance = 4,
}

impl Priority {
    /// Every priority, most urgent first.
    pub const ALL: [Self; 5] = [
        Self::InputCritical,
        Self::VisibleFrame,
        Self::NearFuture,
        Self::Background,
        Self::Maintenance,
    ];

    /// Whether a job at this priority may preempt background work.
    #[must_use]
    pub const fn preempts_background(self) -> bool {
        matches!(self, Self::InputCritical | Self::VisibleFrame)
    }

    /// Whether this work should be throttled when a mobile device is thermally
    /// constrained.
    ///
    /// Interactive work is never throttled: making the editor unresponsive to save
    /// power defeats the purpose. Background work is throttled first, which is what
    /// keeps a long tablet session at a stable frame rate.
    #[must_use]
    pub const fn throttle_under_thermal_pressure(self) -> bool {
        matches!(
            self,
            Self::Background | Self::Maintenance | Self::NearFuture
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordering_puts_interactive_work_first() {
        let mut p = vec![
            Priority::Background,
            Priority::InputCritical,
            Priority::NearFuture,
        ];
        p.sort_unstable();
        assert_eq!(
            p,
            vec![
                Priority::InputCritical,
                Priority::NearFuture,
                Priority::Background
            ]
        );
    }

    #[test]
    fn thermal_throttling_never_touches_interactive_work() {
        assert!(!Priority::InputCritical.throttle_under_thermal_pressure());
        assert!(!Priority::VisibleFrame.throttle_under_thermal_pressure());
        assert!(Priority::Background.throttle_under_thermal_pressure());
    }
}
