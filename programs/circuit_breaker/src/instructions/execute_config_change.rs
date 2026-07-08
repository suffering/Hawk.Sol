use anchor_lang::prelude::*;

use crate::error::BreakerError;
use crate::events::ConfigExecuted;
use crate::ExecuteConfigChange;

/// Guardian-only. Applies a timelocked proposal after `execute_after_ts`.
///
/// Permissionless execute (any signer after timelock) was considered and
/// rejected: it would let a compromised operator schedule griefing by
/// monitoring mempools, and it weakens the separation-of-powers model where
/// governance actions stay guardian-gated. Tradeoff: a malicious or offline
/// guardian can delay execution indefinitely — mitigated by guardian rotation
/// via a prior successful proposal and by `cancel_config_change`.
pub fn process_execute_config_change(ctx: Context<ExecuteConfigChange>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let pending = &ctx.accounts.pending_config;
    require!(now >= pending.execute_after_ts, BreakerError::TimelockNotElapsed);

    let config = &mut ctx.accounts.breaker_config;
    let window_state = &mut ctx.accounts.window_state;
    let old_window_seconds = config.window_seconds;

    pending.apply_to(config);

    // Side effect: changing window_seconds erases in-window outflow history and
    // grants a fresh velocity cap — intentional, so bucket geometry stays valid.
    if config.window_seconds != old_window_seconds {
        window_state.initialize(now);
    }

    emit!(ConfigExecuted {
        breaker_config: config.key(),
    });

    Ok(())
}
