use anchor_lang::prelude::*;
use anchor_spl::token::{set_authority, SetAuthority};

use crate::error::BreakerError;
use crate::state::{BreakerState, AUTHORITY_SEED, BUCKET_COUNT};
use crate::{InitializeBreaker, InitializeBreakerParams};

pub fn process_initialize(ctx: Context<InitializeBreaker>, params: InitializeBreakerParams) -> Result<()> {
    require!(
        params.window_seconds > 0 && params.window_seconds % BUCKET_COUNT as i64 == 0,
        BreakerError::InvalidWindowSeconds
    );
    require!(params.max_bps <= 10_000, BreakerError::InvalidMaxBps);

    let authority_bump = ctx.bumps.breaker_authority;
    let now = Clock::get()?.unix_timestamp;

    let config = &mut ctx.accounts.breaker_config;
    config.guardian = params.guardian;
    config.operator = params.operator;
    config.vault = ctx.accounts.vault.key();
    config.token_mint = ctx.accounts.token_mint.key();
    config.authority_bump = authority_bump;
    config.window_seconds = params.window_seconds;
    config.threshold_mode = params.threshold_mode;
    config.max_absolute = params.max_absolute;
    config.max_bps = params.max_bps;
    config.cooldown_seconds = params.cooldown_seconds;
    config.timelock_floor = params.timelock_floor;
    config.auto_recover = params.auto_recover;
    config.safe_destination = params.safe_destination;
    config.state = BreakerState::Active;
    config.tripped_at = 0;

    ctx.accounts.window_state.initialize(now);

    let vault_key = ctx.accounts.vault.key();
    let seeds: &[&[u8]] = &[AUTHORITY_SEED, vault_key.as_ref(), &[authority_bump]];

    set_authority(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            SetAuthority {
                account_or_mint: ctx.accounts.vault.to_account_info(),
                current_authority: ctx.accounts.vault_authority.to_account_info(),
            },
            &[seeds],
        ),
        anchor_spl::token::spl_token::instruction::AuthorityType::AccountOwner,
        Some(ctx.accounts.breaker_authority.key()),
    )?;

    Ok(())
}
