use std::io::{self, Write};

use anyhow::Result;
use circuit_breaker::state::{BreakerState, ThresholdMode};
use serde::Serialize;

use crate::model::StatusSnapshot;

/// JSON output contract for `solhawk status --json`.
///
/// Stable field names — watch mode and dashboards depend on them.
/// Do not rename without a version bump.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StatusJson {
    pub vault: String,
    pub breaker_config: String,
    pub window_state: String,
    /// `"active"` | `"tripped"` | `"paused"`
    pub state: &'static str,
    pub unix_timestamp: i64,
    pub vault_balance: u64,
    /// `"absolute"` | `"pct_of_balance"` | `"max"`
    pub threshold_mode: &'static str,
    pub max_absolute: u64,
    pub max_bps: u16,
    pub window_seconds: i64,
    pub threshold: u64,
    /// `true` when no outflow cap is configured (`threshold == 0`).
    pub threshold_disabled: bool,
    pub window_used: u64,
    /// 0..=100, capped at assembly time.
    pub utilization_pct: u8,
    pub bucket_roll_in_secs: i64,
    pub tripped_at: i64,
    pub cooldown_seconds: i64,
    pub cooldown_remaining_secs: i64,
    /// Tripped and cooldown elapsed — safe to call `resume`.
    pub resumable: bool,
    pub auto_recover: bool,
}

impl From<&StatusSnapshot> for StatusJson {
    fn from(s: &StatusSnapshot) -> Self {
        Self {
            vault: s.vault.to_string(),
            breaker_config: s.breaker_config.to_string(),
            window_state: s.window_state.to_string(),
            state: breaker_state_json(s.state),
            unix_timestamp: s.unix_timestamp,
            vault_balance: s.vault_balance,
            threshold_mode: threshold_mode_json(s.threshold_mode),
            max_absolute: s.max_absolute,
            max_bps: s.max_bps,
            window_seconds: s.window_seconds,
            threshold: s.threshold,
            threshold_disabled: s.threshold_disabled,
            window_used: s.window_used,
            utilization_pct: s.utilization_pct,
            bucket_roll_in_secs: s.bucket_roll_in_secs,
            tripped_at: s.tripped_at,
            cooldown_seconds: s.cooldown_seconds,
            cooldown_remaining_secs: s.cooldown_remaining_secs,
            resumable: s.resumable,
            auto_recover: s.auto_recover,
        }
    }
}

pub fn to_json_value(snapshot: &StatusSnapshot) -> StatusJson {
    StatusJson::from(snapshot)
}

pub fn print(snapshot: &StatusSnapshot) -> Result<()> {
    let value = StatusJson::from(snapshot);
    serde_json::to_writer(io::stdout(), &value)?;
    writeln!(io::stdout())?;
    Ok(())
}

fn breaker_state_json(state: BreakerState) -> &'static str {
    match state {
        BreakerState::Active => "active",
        BreakerState::Tripped => "tripped",
        BreakerState::Paused => "paused",
    }
}

fn threshold_mode_json(mode: ThresholdMode) -> &'static str {
    match mode {
        ThresholdMode::Absolute => "absolute",
        ThresholdMode::PctOfBalance => "pct_of_balance",
        ThresholdMode::Max => "max",
    }
}
