// SPDX-License-Identifier: GPL-3.0-or-later
use crate::priority::Priority;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::thread::{self, JoinHandle};

type Job = Box<dyn FnOnce() + Send + 'static>;

/// How to size and shape the pool.
#[derive(Debug, Clone, Copy)]
pub struct JobPoolConfig {
    /// Worker thread count. Defaults to available parallelism minus one, leaving a
    /// core for the thread that produces frames.
    pub workers: usize,
    /// Cap on concurrently running background and maintenance work.
    ///
    /// Exists for mobile: under thermal pressure this drops to one so low-priority work
    /// stops competing for the cores that keep the frame rate stable, rather than the
    /// device throttling everything including the user's drag.
    pub max_background_workers: usize,
}

impl Default for JobPoolConfig {
    fn default() -> Self {
        let available = thread::available_parallelism().map_or(4, std::num::NonZeroUsize::get);
        let workers = available.saturating_sub(1).max(1);
        Self {
            workers,
            max_background_workers: workers,
        }
    }
}

#[derive(Default)]
struct Queues {
    /// One deque per priority, indexed by `Priority as usize`.
    lanes: [VecDeque<Job>; 5],
    shutting_down: bool,
}

impl Queues {
    fn pop_highest(&mut self, background_budget: usize, background_running: usize) -> Option<Job> {
        for priority in Priority::ALL {
            let index = priority as usize;
            if priority.throttle_under_thermal_pressure() && background_running >= background_budget
            {
                continue;
            }
            if let Some(lane) = self.lanes.get_mut(index)
                && let Some(job) = lane.pop_front()
            {
                return Some(job);
            }
        }
        None
    }

    fn is_empty(&self) -> bool {
        self.lanes.iter().all(VecDeque::is_empty)
    }
}

/// `Debug` is written by hand because `Queues` holds boxed closures, which cannot
/// derive it. The interesting state is the counters, so those are what it prints.
struct Shared {
    queues: Mutex<Queues>,
    wake: Condvar,
    /// Jobs submitted but not yet finished. Lets `wait_idle` block correctly.
    outstanding: Mutex<usize>,
    idle: Condvar,
    background_running: AtomicUsize,
    background_budget: AtomicUsize,
    stopped: AtomicBool,
}

impl std::fmt::Debug for Shared {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Shared")
            .field(
                "background_running",
                &self.background_running.load(Ordering::Relaxed),
            )
            .field(
                "background_budget",
                &self.background_budget.load(Ordering::Relaxed),
            )
            .field("stopped", &self.stopped.load(Ordering::Relaxed))
            .finish_non_exhaustive()
    }
}

impl Shared {
    fn lock_queues(&self) -> MutexGuard<'_, Queues> {
        // A poisoned lock means a job panicked. The queue itself is still structurally
        // sound, and refusing to schedule anything afterwards would turn one bad job
        // into a dead editor, so recover and carry on.
        self.queues
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn lock_outstanding(&self) -> MutexGuard<'_, usize> {
        self.outstanding
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

/// A priority-scheduled thread pool.
///
/// Deliberately simple. It is the scheduling *policy* that matters — interactive work
/// first, background work throttleable — not clever work stealing. Policy can be
/// tested; cleverness has to be trusted.
#[derive(Debug)]
pub struct JobPool {
    shared: Arc<Shared>,
    workers: Vec<JoinHandle<()>>,
}

impl JobPool {
    /// Starts a pool with the default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(JobPoolConfig::default())
    }

    /// Starts a pool with an explicit configuration.
    #[must_use]
    pub fn with_config(config: JobPoolConfig) -> Self {
        let worker_count = config.workers.max(1);
        let shared = Arc::new(Shared {
            queues: Mutex::new(Queues::default()),
            wake: Condvar::new(),
            outstanding: Mutex::new(0),
            idle: Condvar::new(),
            background_running: AtomicUsize::new(0),
            background_budget: AtomicUsize::new(config.max_background_workers.max(1)),
            stopped: AtomicBool::new(false),
        });

        let workers = (0..worker_count)
            .filter_map(|index| {
                let shared = Arc::clone(&shared);
                thread::Builder::new()
                    .name(format!("topovium-worker-{index}"))
                    .spawn(move || worker_loop(&shared))
                    .map_or_else(
                        |error| {
                            log::error!("failed to spawn worker {index}: {error}");
                            None
                        },
                        Some,
                    )
            })
            .collect();

        Self { shared, workers }
    }

    /// Number of live worker threads.
    #[must_use]
    pub fn worker_count(&self) -> usize {
        self.workers.len()
    }

    /// Queues a job at the given priority.
    ///
    /// Jobs at the same priority run in submission order. Jobs at different priorities
    /// do not: that is the point.
    pub fn submit<F>(&self, priority: Priority, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        if self.shared.stopped.load(Ordering::Acquire) {
            log::warn!("job submitted to a stopped pool; dropping it");
            return;
        }
        *self.shared.lock_outstanding() += 1;
        {
            let mut queues = self.shared.lock_queues();
            if let Some(lane) = queues.lanes.get_mut(priority as usize) {
                lane.push_back(Box::new(job));
            }
        }
        self.shared.wake.notify_one();
    }

    /// Limits how much background and maintenance work runs concurrently.
    ///
    /// Called by the mobile frame budget manager as thermal headroom shrinks. Setting
    /// it to one keeps LOD generation and cache compaction from stealing the cores
    /// that a smooth drag depends on.
    pub fn set_background_budget(&self, budget: usize) {
        self.shared
            .background_budget
            .store(budget.max(1), Ordering::Release);
        self.shared.wake.notify_all();
    }

    /// Blocks until every submitted job has finished. For tests and shutdown.
    pub fn wait_idle(&self) {
        let mut outstanding = self.shared.lock_outstanding();
        while *outstanding > 0 {
            outstanding = self
                .shared
                .idle
                .wait(outstanding)
                .unwrap_or_else(std::sync::PoisonError::into_inner);
        }
    }
}

impl Default for JobPool {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for JobPool {
    fn drop(&mut self) {
        self.shared.stopped.store(true, Ordering::Release);
        {
            let mut queues = self.shared.lock_queues();
            queues.shutting_down = true;
        }
        self.shared.wake.notify_all();
        for worker in self.workers.drain(..) {
            if worker.join().is_err() {
                log::error!("a worker thread panicked; its job did not complete");
            }
        }
    }
}

fn worker_loop(shared: &Arc<Shared>) {
    loop {
        let job = {
            let mut queues = shared.lock_queues();
            loop {
                if queues.shutting_down && queues.is_empty() {
                    return;
                }
                let budget = shared.background_budget.load(Ordering::Acquire);
                let running = shared.background_running.load(Ordering::Acquire);
                if let Some(job) = queues.pop_highest(budget, running) {
                    break job;
                }
                if queues.shutting_down {
                    return;
                }
                queues = shared
                    .wake
                    .wait(queues)
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
            }
        };

        job();

        let mut outstanding = shared.lock_outstanding();
        *outstanding = outstanding.saturating_sub(1);
        if *outstanding == 0 {
            shared.idle.notify_all();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU32;

    #[test]
    fn every_submitted_job_runs() {
        let pool = JobPool::with_config(JobPoolConfig {
            workers: 4,
            max_background_workers: 4,
        });
        let counter = Arc::new(AtomicU32::new(0));
        for _ in 0..200 {
            let counter = Arc::clone(&counter);
            pool.submit(Priority::VisibleFrame, move || {
                counter.fetch_add(1, Ordering::Relaxed);
            });
        }
        pool.wait_idle();
        assert_eq!(counter.load(Ordering::Relaxed), 200);
    }

    #[test]
    fn a_single_worker_drains_higher_priority_lanes_first() {
        // One worker makes ordering deterministic and therefore assertable.
        let pool = JobPool::with_config(JobPoolConfig {
            workers: 1,
            max_background_workers: 1,
        });
        let order = Arc::new(Mutex::new(Vec::new()));

        // Block the worker so everything below queues up behind it.
        let gate = Arc::new((Mutex::new(false), Condvar::new()));
        {
            let gate = Arc::clone(&gate);
            pool.submit(Priority::InputCritical, move || {
                let (lock, cv) = &*gate;
                let mut open = lock
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                while !*open {
                    open = cv
                        .wait(open)
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                }
            });
        }

        for (priority, label) in [
            (Priority::Maintenance, "maintenance"),
            (Priority::Background, "background"),
            (Priority::InputCritical, "input"),
            (Priority::VisibleFrame, "visible"),
        ] {
            let order = Arc::clone(&order);
            pool.submit(priority, move || {
                order
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .push(label);
            });
        }

        {
            let (lock, cv) = &*gate;
            *lock
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = true;
            cv.notify_all();
        }
        pool.wait_idle();

        let observed = order
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        assert_eq!(
            observed,
            vec!["input", "visible", "background", "maintenance"]
        );
    }

    #[test]
    fn submitting_from_inside_a_job_is_allowed() {
        let pool = JobPool::with_config(JobPoolConfig {
            workers: 2,
            max_background_workers: 2,
        });
        let counter = Arc::new(AtomicU32::new(0));
        let inner = Arc::clone(&counter);
        pool.submit(Priority::VisibleFrame, move || {
            inner.fetch_add(1, Ordering::Relaxed);
        });
        pool.wait_idle();
        assert_eq!(counter.load(Ordering::Relaxed), 1);
    }
}
