// SPDX-License-Identifier: GPL-3.0-or-later
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Returned by a job that stopped early because it was cancelled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cancelled;

impl std::fmt::Display for Cancelled {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("job was cancelled")
    }
}

impl std::error::Error for Cancelled {}

/// A cooperative cancellation signal shared between a scheduler and a running job.
///
/// Cooperative, not forced: a job killed mid-write leaves corrupt state, so long jobs
/// are written in chunks and check this between them. A job that never checks is a
/// bug, and the reason `AGENTS.md` requires heavy work to be chunked.
#[derive(Debug, Clone, Default)]
pub struct CancellationToken {
    flag: Arc<AtomicBool>,
}

impl CancellationToken {
    /// Creates a token that has not been cancelled.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Requests cancellation. Idempotent, and safe to call from any thread.
    pub fn cancel(&self) {
        self.flag.store(true, Ordering::Release);
    }

    /// Whether cancellation has been requested.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.flag.load(Ordering::Acquire)
    }

    /// Convenience for the top of a chunk loop: `token.check()?;`
    ///
    /// # Errors
    /// Returns [`Cancelled`] if cancellation has been requested.
    pub fn check(&self) -> Result<(), Cancelled> {
        if self.is_cancelled() {
            Err(Cancelled)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancellation_is_visible_through_every_clone() {
        let token = CancellationToken::new();
        let worker_copy = token.clone();
        assert!(worker_copy.check().is_ok());
        token.cancel();
        assert!(worker_copy.check().is_err());
    }

    #[test]
    fn a_chunked_job_stops_at_the_next_boundary() {
        let token = CancellationToken::new();
        let mut chunks_done = 0;
        for i in 0..100 {
            if token.check().is_err() {
                break;
            }
            chunks_done += 1;
            if i == 9 {
                token.cancel();
            }
        }
        assert_eq!(
            chunks_done, 10,
            "must stop at the boundary after cancellation"
        );
    }
}
