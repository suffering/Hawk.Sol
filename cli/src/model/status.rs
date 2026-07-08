use circuit_breaker::state::{BreakerConfig, BreakerState, ThresholdMode, WindowState};
use circuit_breaker::window::{compute_threshold, effective_window_sum};
use solana_sdk::pubkey::Pubkey;

/// Width of the terminal utilization bar (`render::bar::BAR_WIDTH`).
pub const UTILIZATION_BAR_WIDTH: usize = 10;

/// Frozen view of breaker + window utilization at one on-chain instant.
/// Constructed only by `fetch_status_snapshot` or by hand in tests.
#[derive(Clone, PartialEq, Eq)]
pub struct StatusSnapshot {
    pub vault: Pubkey,
    pub breaker_config: Pubkey,
    pub window_state: Pubkey,
    pub unix_timestamp: i64,
    pub vault_balance: u64,
    pub state: BreakerState,
    pub threshold_mode: ThresholdMode,
    pub max_absolute: u64,
    pub max_bps: u16,
    pub window_seconds: i64,
    pub cooldown_seconds: i64,
    pub auto_recover: bool,
    pub tripped_at: i64,
    pub threshold: u64,
    /// `true` when `threshold == 0` (render uses a dedicated message, not `0 / 0`).
    pub threshold_disabled: bool,
    pub window_used: u64,
    /// 0..=100, capped. When threshold is disabled, 100 if any window usage else 0.
    pub utilization_pct: u8,
    pub bucket_roll_in_secs: i64,
    pub cooldown_remaining_secs: i64,
    pub resumable: bool,
}

/// On-chain inputs gathered in a single RPC batch before assembly.
#[derive(Clone)]
pub struct FetchedInputs {
    pub vault: Pubkey,
    pub breaker_config: Pubkey,
    pub window_state: Pubkey,
    pub vault_balance: u64,
    pub unix_timestamp: i64,
    pub config: BreakerConfig,
    pub window: WindowState,
}

impl StatusSnapshot {
    pub fn from_fetched(inputs: FetchedInputs) -> Self {
        let FetchedInputs {
            vault,
            breaker_config,
            window_state,
            vault_balance,
            unix_timestamp,
            config,
            window,
        } = inputs;

        let threshold = compute_threshold(
            config.threshold_mode,
            config.max_absolute,
            config.max_bps,
            vault_balance,
        );
        let threshold_disabled = threshold == 0;

        let view = effective_window_sum(&window, config.window_seconds, unix_timestamp);
        let window_used = view.window_sum;

        let utilization_pct = compute_utilization_pct(window_used, threshold);

        let bucket_roll_in_secs = view
            .last_bucket_ts
            .saturating_add(view.bucket_duration)
            .saturating_sub(unix_timestamp)
            .max(0);

        let cooldown_remaining_secs = if config.state == BreakerState::Tripped {
            config
                .tripped_at
                .saturating_add(config.cooldown_seconds)
                .saturating_sub(unix_timestamp)
                .max(0)
        } else {
            0
        };

        let resumable = config.state == BreakerState::Tripped && cooldown_remaining_secs == 0;

        Self {
            vault,
            breaker_config,
            window_state,
            unix_timestamp,
            vault_balance,
            state: config.state,
            threshold_mode: config.threshold_mode,
            max_absolute: config.max_absolute,
            max_bps: config.max_bps,
            window_seconds: config.window_seconds,
            cooldown_seconds: config.cooldown_seconds,
            auto_recover: config.auto_recover,
            tripped_at: config.tripped_at,
            threshold,
            threshold_disabled,
            window_used,
            utilization_pct,
            bucket_roll_in_secs,
            cooldown_remaining_secs,
            resumable,
        }
    }
}

/// Utilization percentage capped at 100. Threshold 0 → 100 when used > 0, else 0.
pub fn compute_utilization_pct(window_used: u64, threshold: u64) -> u8 {
    if threshold == 0 {
        return if window_used > 0 { 100 } else { 0 };
    }
    let pct = window_used
        .saturating_mul(100)
        .saturating_div(threshold)
        .min(100);
    pct as u8
}

/// Filled segment count for the utilization bar.
///
/// Rounding rule: **floor** — `utilization_pct * BAR_WIDTH / 100` (integer division).
pub fn bar_fill_count(utilization_pct: u8, bar_width: usize) -> usize {
    (u32::from(utilization_pct) * bar_width as u32 / 100) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bar_fill_uses_floor_rounding() {
        assert_eq!(bar_fill_count(47, UTILIZATION_BAR_WIDTH), 4);
        assert_eq!(bar_fill_count(50, UTILIZATION_BAR_WIDTH), 5);
        assert_eq!(bar_fill_count(100, UTILIZATION_BAR_WIDTH), 10);
        assert_eq!(bar_fill_count(0, UTILIZATION_BAR_WIDTH), 0);
        assert_eq!(bar_fill_count(99, UTILIZATION_BAR_WIDTH), 9);
    }

    #[test]
    fn utilization_pct_caps_at_100() {
        assert_eq!(compute_utilization_pct(200, 100), 100);
        assert_eq!(compute_utilization_pct(0, 0), 0);
        assert_eq!(compute_utilization_pct(1, 0), 100);
    }
}
