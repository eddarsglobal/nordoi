use crate::effect_dispatch::EffectDeliveryKey;

use super::{EffectRetryError, EffectRetryResult};

const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x00000100000001B3;

/// Bounded deterministic retry policy for one external effect delivery stream.
///
/// `max_attempts` includes the first backend execution. Backoff is exponential by powers
/// of two and capped at `max_backoff_ticks`. `jitter_ticks` adds a deterministic offset
/// derived from stable delivery identity and failure ordinal; no ambient randomness is used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectRetryPolicy {
    max_attempts: u32,
    initial_backoff_ticks: u64,
    max_backoff_ticks: u64,
    jitter_ticks: u64,
}

impl EffectRetryPolicy {
    pub fn new(
        max_attempts: u32,
        initial_backoff_ticks: u64,
        max_backoff_ticks: u64,
        jitter_ticks: u64,
    ) -> EffectRetryResult<Self> {
        if max_attempts == 0 {
            return Err(EffectRetryError::InvalidPolicy(
                "max_attempts must be at least 1".into(),
            ));
        }
        if initial_backoff_ticks == 0 {
            return Err(EffectRetryError::InvalidPolicy(
                "initial_backoff_ticks must be at least 1".into(),
            ));
        }
        if max_backoff_ticks < initial_backoff_ticks {
            return Err(EffectRetryError::InvalidPolicy(
                "max_backoff_ticks must be >= initial_backoff_ticks".into(),
            ));
        }
        if jitter_ticks > max_backoff_ticks {
            return Err(EffectRetryError::InvalidPolicy(
                "jitter_ticks must be <= max_backoff_ticks".into(),
            ));
        }

        Ok(Self {
            max_attempts,
            initial_backoff_ticks,
            max_backoff_ticks,
            jitter_ticks,
        })
    }

    pub const fn max_attempts(self) -> u32 {
        self.max_attempts
    }

    pub const fn initial_backoff_ticks(self) -> u64 {
        self.initial_backoff_ticks
    }

    pub const fn max_backoff_ticks(self) -> u64 {
        self.max_backoff_ticks
    }

    pub const fn jitter_ticks(self) -> u64 {
        self.jitter_ticks
    }

    /// Deterministically computes the delay after `failed_attempts` failures.
    /// `failed_attempts` must be >= 1.
    pub(crate) fn delay_ticks(
        self,
        key: EffectDeliveryKey,
        failed_attempts: u32,
    ) -> EffectRetryResult<u64> {
        if failed_attempts == 0 {
            return Err(EffectRetryError::InvalidFailureCount(0));
        }

        let shift = failed_attempts.saturating_sub(1).min(63);
        let factor = 1_u64.checked_shl(shift).unwrap_or(u64::MAX);
        let base = self
            .initial_backoff_ticks
            .saturating_mul(factor)
            .min(self.max_backoff_ticks);

        let jitter = if self.jitter_ticks == 0 {
            0
        } else {
            deterministic_jitter(key, failed_attempts, self.jitter_ticks)
        };

        Ok(base.saturating_add(jitter).min(self.max_backoff_ticks))
    }
}

impl Default for EffectRetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            initial_backoff_ticks: 1,
            max_backoff_ticks: 64,
            jitter_ticks: 3,
        }
    }
}

fn deterministic_jitter(key: EffectDeliveryKey, failed_attempts: u32, limit: u64) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    for byte in key.namespace.0 {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    for byte in key.intent.0.to_le_bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    for byte in failed_attempts.to_le_bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }

    // limit + 1 would overflow for u64::MAX. In that degenerate policy, use the
    // full hash domain directly.
    if limit == u64::MAX {
        hash
    } else {
        hash % (limit + 1)
    }
}
