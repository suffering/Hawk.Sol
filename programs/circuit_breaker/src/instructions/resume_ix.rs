use anchor_lang::prelude::*;

use crate::error::BreakerError;
use crate::state::BreakerState;
use crate::Resume;

pub fn process_resume(ctx: Context<Resume>) -> Result<()> {
    let config = &mut ctx.accounts.breaker_config;

    require!(config.state == BreakerState::Tripped, BreakerError::NotTripped);

    let now = Clock::get()?.unix_timestamp;
    require!(
        now >= config.tripped_at + config.cooldown_seconds,
        BreakerError::CooldownNotElapsed
    );

    if !config.auto_recover {
        require_keys_eq!(
            ctx.accounts.payer.key(),
            config.guardian,
            BreakerError::GuardianRequired
        );
    }

    config.state = BreakerState::Active;

    Ok(())
}
