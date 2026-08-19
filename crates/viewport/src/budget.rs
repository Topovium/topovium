// SPDX-License-Identifier: GPL-3.0-or-later
use std::time::Duration;
use topovium_platform_api::ThermalState;

/// A quality reduction the frame budget manager may apply.
///
/// Ordered by how noticeable each one is. The manager always spends the cheapest
/// remaining lever first, which is why this ordering is part of the type rather than a
/// comment: reducing render resolution before shadow update frequency would be
/// obviously wrong, and the type makes it impossible.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum QualityLever {
    /// Update fewer shadow maps per frame. Nearly invisible in a static scene.
    ShadowUpdateRate = 0,
    /// Lower resolution for distant shadow cascades.
    DistantShadowResolution = 1,
    /// Fewer ambient-occlusion and reflection samples.
    AmbientSampleCount = 2,
    /// More aggressive LOD for objects that are not selected.
    BackgroundLod = 3,
    /// Update screen-space effects every other frame.
    EffectUpdateRate = 4,
    /// Render below native resolution and upscale. The first lever users notice.
    RenderResolution = 5,
    /// Reduce volumetric quality.
    VolumetricQuality = 6,
    /// Cap texture mip level. Visible, and frees memory as well as time.
    TextureMipBias = 7,
    /// Drop the target refresh rate, 120 to 60 to 40. The last resort, because a
    /// stable lower rate beats an unstable higher one.
    RefreshRate = 8,
}

impl QualityLever {
    /// Every lever, least noticeable first.
    pub const ALL: [Self; 9] = [
        Self::ShadowUpdateRate,
        Self::DistantShadowResolution,
        Self::AmbientSampleCount,
        Self::BackgroundLod,
        Self::EffectUpdateRate,
        Self::RenderResolution,
        Self::VolumetricQuality,
        Self::TextureMipBias,
        Self::RefreshRate,
    ];
}

/// How many levers are currently engaged, and what that means for the renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct QualityState {
    engaged: u8,
}

impl QualityState {
    /// Full quality.
    pub const FULL: Self = Self { engaged: 0 };

    /// Whether a given lever is currently applied.
    #[must_use]
    pub fn is_engaged(self, lever: QualityLever) -> bool {
        (lever as u8) < self.engaged
    }

    /// How many levers are applied.
    #[must_use]
    pub const fn engaged_count(self) -> u8 {
        self.engaged
    }

    /// The render resolution scale implied by the engaged levers.
    #[must_use]
    pub fn resolution_scale(self) -> f32 {
        if !self.is_engaged(QualityLever::RenderResolution) {
            return 1.0;
        }
        // Steps down from 1.0 once the resolution lever engages, never below 0.5.
        // Below half resolution, upscaling artefacts cost more than the frames gain.
        let steps = f32::from(self.engaged - QualityLever::RenderResolution as u8);
        (1.0 - steps * 0.1).max(0.5)
    }
}

/// Keeps frames inside their time budget by trading quality for time.
///
/// Deliberately hysteretic. Reacting to a single slow frame produces visible flicker as
/// quality oscillates, so it takes several consecutive overruns to reduce quality and
/// considerably more comfortable frames to restore it. Users notice a change far more
/// than they notice a steady state.
#[derive(Debug)]
pub struct FrameBudget {
    target: Duration,
    quality: QualityState,
    consecutive_over: u32,
    consecutive_under: u32,
    latency_mode: bool,
    thermal_floor: u8,
}

impl FrameBudget {
    /// Overruns required before quality drops.
    pub const DROP_AFTER: u32 = 3;
    /// Comfortable frames required before quality is restored. Higher than
    /// [`Self::DROP_AFTER`] so recovery is cautious and does not oscillate.
    pub const RESTORE_AFTER: u32 = 60;
    /// Fraction of the budget a frame must fit inside to count as comfortable.
    /// Restoring quality at 99% of budget would immediately overrun again.
    pub const COMFORTABLE_FRACTION: f32 = 0.8;

    /// Creates a budget for a refresh rate, defaulting to 60 Hz when unknown.
    #[must_use]
    pub fn for_refresh_rate(hz: Option<f32>) -> Self {
        let hz = hz.filter(|h| *h > 0.0).unwrap_or(60.0);
        Self {
            target: Duration::from_secs_f32(1.0 / hz),
            quality: QualityState::FULL,
            consecutive_over: 0,
            consecutive_under: 0,
            latency_mode: false,
            thermal_floor: 0,
        }
    }

    /// The per-frame time budget.
    #[must_use]
    pub const fn target(&self) -> Duration {
        self.target
    }

    /// Current quality state.
    #[must_use]
    pub const fn quality(&self) -> QualityState {
        self.quality
    }

    /// Records a frame's duration and adjusts quality.
    pub fn record_frame(&mut self, frame_time: Duration) {
        let comfortable = self.target.mul_f32(Self::COMFORTABLE_FRACTION);

        if frame_time > self.target {
            self.consecutive_over += 1;
            self.consecutive_under = 0;
            if self.consecutive_over >= Self::DROP_AFTER {
                self.reduce_quality();
                self.consecutive_over = 0;
            }
        } else if frame_time <= comfortable {
            self.consecutive_under += 1;
            self.consecutive_over = 0;
            if self.consecutive_under >= Self::RESTORE_AFTER {
                self.restore_quality();
                self.consecutive_under = 0;
            }
        } else {
            // Inside budget but not comfortably: hold. This band is what stops the
            // manager hunting up and down around the target.
            self.consecutive_over = 0;
            self.consecutive_under = 0;
        }
    }

    /// Enters or leaves latency mode.
    ///
    /// While the user is dragging, input-to-pixel latency outranks image quality.
    /// Entering the mode engages the cheap levers immediately rather than waiting for
    /// three slow frames, because the first slow frame of a drag is the one that feels
    /// worst.
    pub fn set_latency_mode(&mut self, active: bool) {
        if active && !self.latency_mode {
            self.quality.engaged = self
                .quality
                .engaged
                .max(QualityLever::BackgroundLod as u8 + 1);
        }
        self.latency_mode = active;
        self.consecutive_over = 0;
        self.consecutive_under = 0;
    }

    /// Applies a thermal floor: quality may not be restored above what this state allows.
    ///
    /// Set from the platform's thermal callbacks. This is what keeps a twenty-minute
    /// tablet session steady rather than fast then throttled.
    pub fn set_thermal_state(&mut self, state: ThermalState) {
        self.thermal_floor = match state {
            ThermalState::Nominal => 0,
            ThermalState::Fair => QualityLever::AmbientSampleCount as u8,
            ThermalState::Serious => QualityLever::RenderResolution as u8,
            ThermalState::Critical => QualityLever::TextureMipBias as u8,
        };
        self.quality.engaged = self.quality.engaged.max(self.thermal_floor);
    }

    fn reduce_quality(&mut self) {
        let max = QualityLever::ALL.len() as u8;
        self.quality.engaged = (self.quality.engaged + 1).min(max);
    }

    fn restore_quality(&mut self) {
        let floor = if self.latency_mode {
            self.thermal_floor
                .max(QualityLever::BackgroundLod as u8 + 1)
        } else {
            self.thermal_floor
        };
        self.quality.engaged = self.quality.engaged.saturating_sub(1).max(floor);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    #[test]
    fn refresh_rate_determines_the_budget() {
        assert_eq!(
            FrameBudget::for_refresh_rate(Some(120.0)).target(),
            Duration::from_secs_f32(1.0 / 120.0)
        );
        assert_eq!(
            FrameBudget::for_refresh_rate(None).target(),
            Duration::from_secs_f32(1.0 / 60.0)
        );
        assert_eq!(
            FrameBudget::for_refresh_rate(Some(0.0)).target(),
            Duration::from_secs_f32(1.0 / 60.0)
        );
    }

    #[test]
    fn one_slow_frame_does_not_change_quality() {
        // Reacting to a single hitch is what makes quality visibly flicker.
        let mut budget = FrameBudget::for_refresh_rate(Some(60.0));
        budget.record_frame(ms(30));
        assert_eq!(budget.quality(), QualityState::FULL);
    }

    #[test]
    fn sustained_overrun_engages_the_least_noticeable_lever_first() {
        let mut budget = FrameBudget::for_refresh_rate(Some(60.0));
        for _ in 0..FrameBudget::DROP_AFTER {
            budget.record_frame(ms(30));
        }
        assert!(budget.quality().is_engaged(QualityLever::ShadowUpdateRate));
        assert!(!budget.quality().is_engaged(QualityLever::RenderResolution));
    }

    #[test]
    fn levers_engage_in_order_of_how_noticeable_they_are() {
        let mut budget = FrameBudget::for_refresh_rate(Some(60.0));
        for _ in 0..(FrameBudget::DROP_AFTER * 6) {
            budget.record_frame(ms(30));
        }
        let quality = budget.quality();
        assert!(quality.is_engaged(QualityLever::ShadowUpdateRate));
        assert!(quality.is_engaged(QualityLever::RenderResolution));
        assert!(
            !quality.is_engaged(QualityLever::RefreshRate),
            "refresh rate is the last resort"
        );
    }

    #[test]
    fn recovery_is_slower_than_degradation() {
        let mut budget = FrameBudget::for_refresh_rate(Some(60.0));
        for _ in 0..FrameBudget::DROP_AFTER {
            budget.record_frame(ms(30));
        }
        let degraded = budget.quality().engaged_count();

        for _ in 0..(FrameBudget::RESTORE_AFTER - 1) {
            budget.record_frame(ms(8));
        }
        assert_eq!(
            budget.quality().engaged_count(),
            degraded,
            "must not restore early"
        );

        budget.record_frame(ms(8));
        assert_eq!(budget.quality().engaged_count(), degraded - 1);
    }

    #[test]
    fn frames_just_inside_budget_neither_degrade_nor_restore() {
        let mut budget = FrameBudget::for_refresh_rate(Some(60.0));
        for _ in 0..200 {
            budget.record_frame(ms(15)); // under 16.67 but over the 13.3 comfort line
        }
        assert_eq!(budget.quality(), QualityState::FULL);
    }

    #[test]
    fn latency_mode_trades_quality_for_responsiveness_immediately() {
        let mut budget = FrameBudget::for_refresh_rate(Some(120.0));
        budget.set_latency_mode(true);
        assert!(budget.quality().is_engaged(QualityLever::BackgroundLod));
        assert!(!budget.quality().is_engaged(QualityLever::RenderResolution));
    }

    #[test]
    fn thermal_pressure_prevents_quality_from_being_restored() {
        // The whole point of sustained mode: do not climb back up and throttle again.
        let mut budget = FrameBudget::for_refresh_rate(Some(60.0));
        budget.set_thermal_state(ThermalState::Serious);
        let floor = budget.quality().engaged_count();
        assert!(floor > 0);

        for _ in 0..(FrameBudget::RESTORE_AFTER * 5) {
            budget.record_frame(ms(4));
        }
        assert_eq!(budget.quality().engaged_count(), floor);
    }

    #[test]
    fn resolution_never_drops_below_half() {
        let mut budget = FrameBudget::for_refresh_rate(Some(60.0));
        for _ in 0..(FrameBudget::DROP_AFTER * 20) {
            budget.record_frame(ms(200));
        }
        assert!(budget.quality().resolution_scale() >= 0.5);
    }
}
