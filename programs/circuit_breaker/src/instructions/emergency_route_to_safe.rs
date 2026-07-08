use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer};

use crate::error::BreakerError;
use crate::events::EmergencyRoute;
use crate::state::{BreakerState, AUTHORITY_SEED};
use crate::EmergencyRouteToSafe;

/// Guardian evacuation to the immutable `safe_destination` while Tripped.
///
/// Does not call `advance_and_check` / `advance_buckets`: §5 emergency routing
/// bypasses the velocity window (guardian recovery, not operator outflow).
/// Requires `Tripped` — no Active-state window bypass; guardian must trip first.
pub fn process_emergency_route_to_safe(
    ctx: Context<EmergencyRouteToSafe>,
    amount: u64,
) -> Result<()> {
    let config = &ctx.accounts.breaker_config;
    require!(
        config.state == BreakerState::Tripped,
        BreakerError::EmergencyRouteRequiresTripped
    );

    let vault_key = config.vault;
    let authority_bump = config.authority_bump;
    let seeds: &[&[u8]] = &[AUTHORITY_SEED, vault_key.as_ref(), &[authority_bump]];

    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault.to_account_info(),
                to: ctx.accounts.safe_destination.to_account_info(),
                authority: ctx.accounts.breaker_authority.to_account_info(),
            },
            &[seeds],
        ),
        amount,
    )?;

    emit!(EmergencyRoute {
        amount,
        destination: ctx.accounts.safe_destination.key(),
    });

    Ok(())
}
