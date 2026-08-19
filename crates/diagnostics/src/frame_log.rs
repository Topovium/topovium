// SPDX-License-Identifier: GPL-3.0-or-later
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// One frame's measurements.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FrameSample {
    /// Wall time the CPU spent producing this frame.
    pub cpu: Duration,
    /// Wall time the GPU spent, from timestamp queries. `None` where the backend does
    /// not support them, which is common on mobile.
    pub gpu: Option<Duration>,
    /// Time from the input sample that drove this frame to it being presented. This
    /// is the number users actually feel, and the one most engines never report.
    pub input_to_pixel: Option<Duration>,
}

impl FrameSample {
    /// A CPU-only sample.
    #[must_use]
    pub const fn cpu_only(cpu: Duration) -> Self {
        Self {
            cpu,
            gpu: None,
            input_to_pixel: None,
        }
    }
}

/// p50, p95, and p99 for one metric.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Percentiles {
    pub p50: Duration,
    pub p95: Duration,
    pub p99: Duration,
    /// The worst frame observed. Reported because a single 200 ms stall is a bug even
    /// when p99 looks healthy.
    pub max: Duration,
    /// How many samples these were computed from.
    pub sample_count: usize,
}

/// A fixed-capacity ring of recent frames.
///
/// Fixed capacity is the point: the HUD must never be the thing that allocates during
/// a drag, and an unbounded history of a long session is a memory leak with a chart.
#[derive(Debug)]
pub struct FrameLog {
    samples: Vec<FrameSample>,
    capacity: usize,
    next: usize,
    stall_threshold: Duration,
    stalls: u32,
}

impl FrameLog {
    /// Frames retained by default: about eight seconds at 120 Hz.
    pub const DEFAULT_CAPACITY: usize = 1024;

    /// A CPU frame longer than this counts as a stall.
    ///
    /// 50 ms comes from the design target: no single UI stall longer than 50 ms during
    /// typical modelling.
    pub const DEFAULT_STALL_THRESHOLD: Duration = Duration::from_millis(50);

    /// Creates a log holding `capacity` frames. A capacity of zero is raised to one so
    /// the ring arithmetic stays valid.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        let capacity = capacity.max(1);
        Self {
            samples: Vec::with_capacity(capacity),
            capacity,
            next: 0,
            stall_threshold: Self::DEFAULT_STALL_THRESHOLD,
            stalls: 0,
        }
    }

    /// Overrides the stall threshold.
    #[must_use]
    pub const fn with_stall_threshold(mut self, threshold: Duration) -> Self {
        self.stall_threshold = threshold;
        self
    }

    /// Records a frame, evicting the oldest once full.
    pub fn record(&mut self, sample: FrameSample) {
        if sample.cpu >= self.stall_threshold {
            self.stalls = self.stalls.saturating_add(1);
        }
        if self.samples.len() < self.capacity {
            self.samples.push(sample);
        } else if let Some(slot) = self.samples.get_mut(self.next) {
            *slot = sample;
        }
        self.next = (self.next + 1) % self.capacity;
    }

    /// How many recorded frames exceeded the stall threshold.
    #[must_use]
    pub const fn stall_count(&self) -> u32 {
        self.stalls
    }

    /// Number of retained samples.
    #[must_use]
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    /// Whether nothing has been recorded.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// CPU frame time percentiles, or `None` if nothing has been recorded.
    #[must_use]
    pub fn cpu_percentiles(&self) -> Option<Percentiles> {
        self.percentiles_of(|s| Some(s.cpu))
    }

    /// GPU frame time percentiles over samples that carry a GPU timing.
    #[must_use]
    pub fn gpu_percentiles(&self) -> Option<Percentiles> {
        self.percentiles_of(|s| s.gpu)
    }

    /// Input-to-pixel percentiles over samples that carry a latency measurement.
    #[must_use]
    pub fn latency_percentiles(&self) -> Option<Percentiles> {
        self.percentiles_of(|s| s.input_to_pixel)
    }

    fn percentiles_of(
        &self,
        extract: impl Fn(&FrameSample) -> Option<Duration>,
    ) -> Option<Percentiles> {
        let mut values: Vec<Duration> = self.samples.iter().filter_map(extract).collect();
        if values.is_empty() {
            return None;
        }
        values.sort_unstable();
        Some(Percentiles {
            p50: nearest_rank(&values, 50.0),
            p95: nearest_rank(&values, 95.0),
            p99: nearest_rank(&values, 99.0),
            max: *values.last()?,
            sample_count: values.len(),
        })
    }

    /// Clears samples and the stall counter, keeping allocated capacity.
    pub fn reset(&mut self) {
        self.samples.clear();
        self.next = 0;
        self.stalls = 0;
    }
}

impl Default for FrameLog {
    fn default() -> Self {
        Self::with_capacity(Self::DEFAULT_CAPACITY)
    }
}

/// Nearest-rank percentile over a sorted, non-empty slice.
///
/// Nearest-rank rather than interpolated: a reported p99 should be a frame time that
/// actually occurred, so it can be traced back to a specific frame in a capture.
fn nearest_rank(sorted: &[Duration], percentile: f64) -> Duration {
    debug_assert!(!sorted.is_empty());
    let len = sorted.len();
    let rank = (percentile / 100.0 * len as f64).ceil() as usize;
    let index = rank.saturating_sub(1).min(len - 1);
    sorted.get(index).copied().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    #[test]
    fn percentiles_report_the_tail_not_the_average() {
        // 99 fast frames and one 200 ms hitch. A mean would read ~10 ms and hide it.
        let mut log = FrameLog::with_capacity(128);
        for _ in 0..99 {
            log.record(FrameSample::cpu_only(ms(8)));
        }
        log.record(FrameSample::cpu_only(ms(200)));

        let p = log.cpu_percentiles().unwrap();
        assert_eq!(p.p50, ms(8));
        assert_eq!(p.max, ms(200));
        assert_eq!(p.sample_count, 100);
    }

    #[test]
    fn ring_evicts_oldest_and_never_grows() {
        let mut log = FrameLog::with_capacity(4);
        for i in 1..=10 {
            log.record(FrameSample::cpu_only(ms(i)));
        }
        assert_eq!(log.len(), 4, "capacity must be respected");
        let p = log.cpu_percentiles().unwrap();
        assert_eq!(
            p.max,
            ms(10),
            "most recent frames should be the retained ones"
        );
    }

    #[test]
    fn stalls_are_counted_against_the_fifty_millisecond_target() {
        let mut log = FrameLog::with_capacity(16);
        log.record(FrameSample::cpu_only(ms(8)));
        log.record(FrameSample::cpu_only(ms(60)));
        log.record(FrameSample::cpu_only(ms(49)));
        log.record(FrameSample::cpu_only(ms(50)));
        assert_eq!(log.stall_count(), 2);
    }

    #[test]
    fn gpu_percentiles_ignore_samples_without_gpu_timings() {
        let mut log = FrameLog::with_capacity(8);
        log.record(FrameSample::cpu_only(ms(8)));
        log.record(FrameSample {
            cpu: ms(8),
            gpu: Some(ms(5)),
            input_to_pixel: None,
        });
        let p = log.gpu_percentiles().unwrap();
        assert_eq!(p.sample_count, 1);
        assert_eq!(p.p50, ms(5));
    }

    #[test]
    fn empty_log_reports_no_percentiles_rather_than_zero() {
        let log = FrameLog::default();
        assert!(log.cpu_percentiles().is_none());
    }
}
