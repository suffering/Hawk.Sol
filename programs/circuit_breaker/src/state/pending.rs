use anchor_lang::prelude::*;

use super::ThresholdMode;

/// Timelock-governed config delta. Fields absent here are immutable on-chain
/// (`safe_destination`, `timelock_floor`).
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub struct ProposedConfigParams {
    pub guardian: Pubkey,
    pub operator: Pubkey,
    pub window_seconds: i64,
    pub threshold_mode: ThresholdMode,
    pub max_absolute: u64,
    pub max_bps: u16,
    pub cooldown_seconds: i64,
    pub auto_recover: bool,
}

#[account]
#[derive(InitSpace)]
pub struct PendingConfigChange {
    pub breaker_config: Pubkey,
    pub vault: Pubkey,
    pub proposed: ProposedConfigParams,
    pub execute_after_ts: i64,
    pub effective_delay_seconds: i64,
    pub proposed_at: i64,
    pub bump: u8,
}

impl PendingConfigChange {
    pub fn apply_to(&self, config: &mut super::BreakerConfig) {
        config.guardian = self.proposed.guardian;
        config.operator = self.proposed.operator;
        config.window_seconds = self.proposed.window_seconds;
        config.threshold_mode = self.proposed.threshold_mode;
        config.max_absolute = self.proposed.max_absolute;
        config.max_bps = self.proposed.max_bps;
        config.cooldown_seconds = self.proposed.cooldown_seconds;
        config.auto_recover = self.proposed.auto_recover;
    }
}
