use anchor_lang::prelude::*;

use crate::events::Tripped;
use crate::state::BreakerState;
use crate::Trip;

pub fn process_trip(ctx: Context<Trip>) -> Result<()> {
    let config = &mut ctx.accounts.breaker_config;
    let now = Clock::get()?.unix_timestamp;

    config.state = BreakerState::Tripped;
    config.tripped_at = now;

    emit!(Tripped {
        tripped_at: now,
        window_sum: 0,
        attempted_amount: 0,
        threshold: 0,
        manual: true,
    });

    Ok(())
}
