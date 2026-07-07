use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer};

use crate::error::BreakerError;
use crate::events::{Tripped, Withdraw};
use crate::state::{BreakerState, AUTHORITY_SEED};
use crate::window::{advance_and_check, compute_threshold};
use crate::GuardedWithdraw;

pub fn process_guarded_withdraw(
    ctx: Context<GuardedWithdraw>,
    amount: u64,
    _destination_key: Pubkey,
) -> Result<()> {
    let config = &mut ctx.accounts.breaker_config;

    match config.state {
        BreakerState::Active => {}
        BreakerState::Tripped => return err!(BreakerError::Tripped),
        BreakerState::Paused => return err!(BreakerError::Paused),
    }

    let now = Clock::get()?.unix_timestamp;
    let vault_balance = ctx.accounts.vault.amount;
    let threshold = compute_threshold(
        config.threshold_mode,
        config.max_absolute,
        config.max_bps,
        vault_balance,
    );

    let result = advance_and_check(
        &mut ctx.accounts.window_state,
        config.window_seconds,
        now,
        amount,
        threshold,
    );

    if result.tripped {
        config.state = BreakerState::Tripped;
        config.tripped_at = now;
        emit!(Tripped {
            tripped_at: now,
            window_sum: result.window_sum,
            attempted_amount: amount,
            threshold: result.threshold,
            manual: false,
        });
        // Return Ok so trip state persists (a failing instruction would roll back the trip).
        return Ok(());
    }

    // Phase 2: destination allowlist / heightened scrutiny for never-seen destinations.

    let vault_key = config.vault;
    let authority_bump = config.authority_bump;
    let seeds: &[&[u8]] = &[AUTHORITY_SEED, vault_key.as_ref(), &[authority_bump]];

    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault.to_account_info(),
                to: ctx.accounts.destination.to_account_info(),
                authority: ctx.accounts.breaker_authority.to_account_info(),
            },
            &[seeds],
        ),
        amount,
    )?;

    emit!(Withdraw {
        amount,
        destination: ctx.accounts.destination.key(),
        window_sum: result.window_sum,
        threshold: result.threshold,
    });

    Ok(())
}
