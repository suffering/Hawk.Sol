use anchor_lang::prelude::*;

use crate::error::BreakerError;
use crate::events::ConfigProposed;
use crate::state::{PendingConfigChange, ProposedConfigParams};
use crate::ProposeConfigChange;

use super::validate::validate_proposed_config;

/// Guardian-only. Creates the per-breaker `PendingConfigChange` PDA.
///
/// `requested_delay` is clamped up to `breaker_config.timelock_floor`.
/// There is no upper bound on `requested_delay`; a mistaken long delay is
/// guardian-recoverable via `cancel_config_change`.
pub fn process_propose_config_change(
    ctx: Context<ProposeConfigChange>,
    new_params: ProposedConfigParams,
    requested_delay: i64,
) -> Result<()> {
    require!(requested_delay >= 0, BreakerError::InvalidRequestedDelay);
    validate_proposed_config(&new_params)?;

    let now = Clock::get()?.unix_timestamp;
    let config = &ctx.accounts.breaker_config;
    let effective_delay = requested_delay.max(config.timelock_floor);
    let execute_after_ts = now
        .checked_add(effective_delay)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    let pending = &mut ctx.accounts.pending_config;
    pending.set_inner(PendingConfigChange {
        breaker_config: config.key(),
        vault: config.vault,
        proposed: new_params,
        execute_after_ts,
        effective_delay_seconds: effective_delay,
        proposed_at: now,
        bump: ctx.bumps.pending_config,
    });

    emit!(ConfigProposed {
        breaker_config: config.key(),
        execute_after_ts,
        effective_delay_seconds: effective_delay,
    });

    Ok(())
}
