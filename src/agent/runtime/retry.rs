use std::time::Duration;

/// A simple backoff strategy enum. Implementations are deterministic and
/// do not sleep — they only compute backoff ticks to be used by schedulers.
#[derive(Debug, Clone, Copy)]
pub enum BackoffStrategy {
    Fixed { millis: u64 },
    Exponential { base_millis: u64, multiplier: u32 },
}

impl BackoffStrategy {
    pub fn calculate(&self, attempt: u32) -> Duration {
        match *self {
            BackoffStrategy::Fixed { millis } => Duration::from_millis(millis),
            BackoffStrategy::Exponential {
                base_millis,
                multiplier,
            } => {
                let factor = multiplier.saturating_pow(attempt.saturating_sub(1));
                let millis = base_millis.saturating_mul(factor as u64);
                Duration::from_millis(millis)
            }
        }
    }
}

/// Decision returned by the RetryManager when a failure occurs.
#[derive(Debug, Clone)]
pub enum RetryDecision {
    Retry { next_backoff: Duration },
    Fail,
}

/// Context passed to the RetryManager to decide retries.
pub struct RetryContext {
    pub step_id: String,
    pub attempts: u32,
    pub max_attempts: u32,
}

/// Production-ready RetryManager that encapsulates retry policy and backoff
/// calculation. It is deterministic and side-effect free (no sleeping).
pub struct RetryManager {
    pub max_attempts: u32,
    pub strategy: BackoffStrategy,
}

impl RetryManager {
    pub fn new(max_attempts: u32) -> Self {
        Self {
            max_attempts,
            strategy: BackoffStrategy::Exponential {
                base_millis: 100,
                multiplier: 2,
            },
        }
    }

    pub fn with_strategy(mut self, strat: BackoffStrategy) -> Self {
        self.strategy = strat;
        self
    }

    /// Maintain compatibility: simple boolean check.
    pub fn should_retry(&self, attempts: u32) -> bool {
        attempts < self.max_attempts
    }

    /// Decide whether to retry given a context. Returns `RetryDecision`.
    pub fn decide(&self, ctx: &RetryContext) -> RetryDecision {
        if ctx.attempts < ctx.max_attempts && ctx.attempts < self.max_attempts {
            let backoff = self.strategy.calculate(ctx.attempts + 1);
            RetryDecision::Retry {
                next_backoff: backoff,
            }
        } else {
            RetryDecision::Fail
        }
    }
}
