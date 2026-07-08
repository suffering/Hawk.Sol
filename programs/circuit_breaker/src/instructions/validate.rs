use anchor_lang::prelude::*;

use crate::error::BreakerError;
use crate::state::{BUCKET_COUNT, ProposedConfigParams};

pub fn validate_mutable_config_params(window_seconds: i64, max_bps: u16) -> Result<()> {
    require!(
        window_seconds > 0 && window_seconds % BUCKET_COUNT as i64 == 0,
        BreakerError::InvalidWindowSeconds
    );
    require!(max_bps <= 10_000, BreakerError::InvalidMaxBps);
    Ok(())
}

pub fn validate_proposed_config(params: &ProposedConfigParams) -> Result<()> {
    validate_mutable_config_params(params.window_seconds, params.max_bps)
}
