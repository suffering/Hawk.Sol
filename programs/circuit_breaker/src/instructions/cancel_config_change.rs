use anchor_lang::prelude::*;

use crate::events::ConfigCancelled;
use crate::CancelConfigChange;

pub fn process_cancel_config_change(ctx: Context<CancelConfigChange>) -> Result<()> {
    emit!(ConfigCancelled {
        breaker_config: ctx.accounts.breaker_config.key(),
    });
    Ok(())
}
