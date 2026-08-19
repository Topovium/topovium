// SPDX-License-Identifier: GPL-3.0-or-later
/// How hot the device is.
///
/// Mobile GPUs sustain peak clocks for a minute or two, then throttle. Reacting after
/// the throttle produces a visible cliff; reacting to rising headroom pressure keeps a
/// twenty-minute session smooth. That is the difference this enum exists to enable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ThermalState {
    /// Full quality.
    Nominal,
    /// Begin reducing background work and non-essential effects.
    Fair,
    /// Reduce render resolution and shadow quality.
    Serious,
    /// Minimum viable quality. Editing must still work.
    Critical,
}

impl ThermalState {
    /// A multiplier on the render resolution scale for this state.
    ///
    /// Gradual steps, never a preset cliff from Ultra to Low: users notice a sudden
    /// change far more than a slow one.
    #[must_use]
    pub const fn resolution_scale(self) -> f32 {
        match self {
            Self::Nominal => 1.0,
            Self::Fair => 0.9,
            Self::Serious => 0.75,
            Self::Critical => 0.6,
        }
    }

    /// How many workers may run background work in this state.
    #[must_use]
    pub const fn background_worker_budget(self, total_workers: usize) -> usize {
        match self {
            Self::Nominal => total_workers,
            Self::Fair => total_workers / 2,
            Self::Serious | Self::Critical => 1,
        }
    }
}

/// How urgently the system wants memory back.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MemoryPressure {
    Normal,
    /// Release rebuildable caches now.
    Warning,
    /// Release everything not needed for the current frame. The next stage is the
    /// operating system killing the process.
    Critical,
}

/// Something the operating system did to the application.
#[derive(Debug, Clone, PartialEq)]
pub enum LifecycleEvent {
    /// The surface became available or changed size.
    SurfaceReady {
        width: u32,
        height: u32,
    },
    /// The surface is about to be destroyed. All GPU resources referencing it must go.
    SurfaceLost,
    /// The application moved to the background.
    ///
    /// On mobile this is the last reliable moment before the process may be killed
    /// without further warning, so the project is committed here — not on a timer.
    Backgrounded,
    Foregrounded,
    ThermalChanged(ThermalState),
    MemoryPressure(MemoryPressure),
    /// The user asked to quit. The shell waits for the transaction to commit.
    QuitRequested,
}

impl LifecycleEvent {
    /// Whether this event obliges an immediate, synchronous save.
    #[must_use]
    pub const fn requires_immediate_commit(&self) -> bool {
        matches!(self, Self::Backgrounded | Self::QuitRequested)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quality_degrades_gradually_rather_than_in_a_cliff() {
        let scales: Vec<f32> = [
            ThermalState::Nominal,
            ThermalState::Fair,
            ThermalState::Serious,
            ThermalState::Critical,
        ]
        .map(ThermalState::resolution_scale)
        .to_vec();

        for pair in scales.windows(2) {
            let (before, after) = (pair[0], pair[1]);
            assert!(after < before, "each step must reduce quality");
            assert!(before - after <= 0.2, "no step may drop more than 20%");
        }
    }

    #[test]
    fn background_work_is_throttled_before_the_device_throttles_itself() {
        assert_eq!(ThermalState::Nominal.background_worker_budget(8), 8);
        assert_eq!(ThermalState::Fair.background_worker_budget(8), 4);
        assert_eq!(ThermalState::Critical.background_worker_budget(8), 1);
    }

    #[test]
    fn backgrounding_forces_a_commit_because_the_process_may_not_return() {
        assert!(LifecycleEvent::Backgrounded.requires_immediate_commit());
        assert!(!LifecycleEvent::Foregrounded.requires_immediate_commit());
    }
}
