use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace, Debug)]
pub enum BreakerState {
    Active,
    Tripped,
    Paused,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum ThresholdMode {
    Absolute,
    PctOfBalance,
    Max,
}

#[account]
#[derive(InitSpace)]
pub struct BreakerConfig {
    pub guardian: Pubkey,
    pub operator: Pubkey,
    pub vault: Pubkey,
    pub token_mint: Pubkey,
    pub authority_bump: u8,
    pub window_seconds: i64,
    pub threshold_mode: ThresholdMode,
    pub max_absolute: u64,
    pub max_bps: u16,
    pub cooldown_seconds: i64,
    pub timelock_floor: i64,
    pub auto_recover: bool,
    pub safe_destination: Pubkey,
    pub state: BreakerState,
    pub tripped_at: i64,
}
