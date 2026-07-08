pub mod error;
pub mod events;
pub mod instructions;
pub mod state;
pub mod window;

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::error::BreakerError;
use crate::state::{
    BreakerConfig, PendingConfigChange, WindowState, AUTHORITY_SEED, BREAKER_CONFIG_SEED,
    PENDING_CONFIG_SEED, WINDOW_STATE_SEED,
};

pub use crate::state::ProposedConfigParams;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitializeBreakerParams {
    pub guardian: Pubkey,
    pub operator: Pubkey,
    pub window_seconds: i64,
    pub threshold_mode: crate::state::ThresholdMode,
    pub max_absolute: u64,
    pub max_bps: u16,
    pub cooldown_seconds: i64,
    pub timelock_floor: i64,
    pub auto_recover: bool,
    pub safe_destination: Pubkey,
}

#[derive(Accounts)]
pub struct InitializeBreaker<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub vault_authority: Signer<'info>,

    pub token_mint: Account<'info, Mint>,

    #[account(
        mut,
        constraint = vault.mint == token_mint.key() @ BreakerError::MintMismatch,
        constraint = vault.owner == vault_authority.key(),
    )]
    pub vault: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = payer,
        space = 8 + BreakerConfig::INIT_SPACE,
        seeds = [BREAKER_CONFIG_SEED, vault.key().as_ref()],
        bump,
    )]
    pub breaker_config: Account<'info, BreakerConfig>,

    #[account(
        init,
        payer = payer,
        space = 8 + WindowState::INIT_SPACE,
        seeds = [WINDOW_STATE_SEED, vault.key().as_ref()],
        bump,
    )]
    pub window_state: Account<'info, WindowState>,

    /// CHECK: Breaker authority PDA; becomes vault token authority.
    #[account(
        seeds = [AUTHORITY_SEED, vault.key().as_ref()],
        bump,
    )]
    pub breaker_authority: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(amount: u64, destination_key: Pubkey)]
pub struct GuardedWithdraw<'info> {
    pub operator: Signer<'info>,

    #[account(
        mut,
        seeds = [BREAKER_CONFIG_SEED, breaker_config.vault.as_ref()],
        bump,
        constraint = breaker_config.operator == operator.key() @ BreakerError::UnauthorizedOperator,
    )]
    pub breaker_config: Account<'info, BreakerConfig>,

    #[account(
        mut,
        seeds = [WINDOW_STATE_SEED, breaker_config.vault.as_ref()],
        bump,
    )]
    pub window_state: Account<'info, WindowState>,

    #[account(
        mut,
        constraint = vault.key() == breaker_config.vault,
        constraint = vault.mint == breaker_config.token_mint,
    )]
    pub vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = destination.key() == destination_key @ BreakerError::DestinationMismatch,
        constraint = destination.mint == breaker_config.token_mint,
    )]
    pub destination: Account<'info, TokenAccount>,

    /// CHECK: Breaker authority PDA — sole signer for vault transfers.
    #[account(
        seeds = [AUTHORITY_SEED, breaker_config.vault.as_ref()],
        bump = breaker_config.authority_bump,
    )]
    pub breaker_authority: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Trip<'info> {
    pub guardian: Signer<'info>,

    #[account(
        mut,
        seeds = [BREAKER_CONFIG_SEED, breaker_config.vault.as_ref()],
        bump,
        constraint = breaker_config.guardian == guardian.key() @ BreakerError::UnauthorizedGuardian,
    )]
    pub breaker_config: Account<'info, BreakerConfig>,
}

#[derive(Accounts)]
pub struct Resume<'info> {
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [BREAKER_CONFIG_SEED, breaker_config.vault.as_ref()],
        bump,
    )]
    pub breaker_config: Account<'info, BreakerConfig>,
}

#[derive(Accounts)]
pub struct ProposeConfigChange<'info> {
    pub guardian: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        seeds = [BREAKER_CONFIG_SEED, breaker_config.vault.as_ref()],
        bump,
        constraint = breaker_config.guardian == guardian.key() @ BreakerError::UnauthorizedGuardian,
    )]
    pub breaker_config: Account<'info, BreakerConfig>,

    #[account(
        init,
        payer = payer,
        space = 8 + PendingConfigChange::INIT_SPACE,
        seeds = [PENDING_CONFIG_SEED, breaker_config.vault.as_ref()],
        bump,
    )]
    pub pending_config: Account<'info, PendingConfigChange>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ExecuteConfigChange<'info> {
    pub guardian: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [BREAKER_CONFIG_SEED, breaker_config.vault.as_ref()],
        bump,
        constraint = breaker_config.guardian == guardian.key() @ BreakerError::UnauthorizedGuardian,
    )]
    pub breaker_config: Account<'info, BreakerConfig>,

    #[account(
        mut,
        seeds = [WINDOW_STATE_SEED, breaker_config.vault.as_ref()],
        bump,
    )]
    pub window_state: Account<'info, WindowState>,

    #[account(
        mut,
        close = payer,
        seeds = [PENDING_CONFIG_SEED, breaker_config.vault.as_ref()],
        bump = pending_config.bump,
        constraint = pending_config.breaker_config == breaker_config.key() @ BreakerError::NoPendingConfig,
    )]
    pub pending_config: Account<'info, PendingConfigChange>,
}

#[derive(Accounts)]
pub struct CancelConfigChange<'info> {
    pub guardian: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        seeds = [BREAKER_CONFIG_SEED, breaker_config.vault.as_ref()],
        bump,
        constraint = breaker_config.guardian == guardian.key() @ BreakerError::UnauthorizedGuardian,
    )]
    pub breaker_config: Account<'info, BreakerConfig>,

    #[account(
        mut,
        close = payer,
        seeds = [PENDING_CONFIG_SEED, breaker_config.vault.as_ref()],
        bump = pending_config.bump,
        constraint = pending_config.breaker_config == breaker_config.key() @ BreakerError::NoPendingConfig,
    )]
    pub pending_config: Account<'info, PendingConfigChange>,
}

#[derive(Accounts)]
pub struct EmergencyRouteToSafe<'info> {
    pub guardian: Signer<'info>,

    #[account(
        seeds = [BREAKER_CONFIG_SEED, breaker_config.vault.as_ref()],
        bump,
        constraint = breaker_config.guardian == guardian.key() @ BreakerError::UnauthorizedGuardian,
    )]
    pub breaker_config: Account<'info, BreakerConfig>,

    #[account(
        mut,
        constraint = vault.key() == breaker_config.vault,
        constraint = vault.mint == breaker_config.token_mint,
    )]
    pub vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = safe_destination.key() == breaker_config.safe_destination @ BreakerError::DestinationMismatch,
        constraint = safe_destination.mint == breaker_config.token_mint,
    )]
    pub safe_destination: Account<'info, TokenAccount>,

    /// CHECK: Breaker authority PDA — sole signer for vault transfers.
    #[account(
        seeds = [AUTHORITY_SEED, breaker_config.vault.as_ref()],
        bump = breaker_config.authority_bump,
    )]
    pub breaker_authority: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
}

declare_id!("7Wuw9J1cW8V2R1KAA6T3y3bRDpBYJ5FkQWMj8gDKb1MW");

#[program]
pub mod circuit_breaker {
    use super::*;

    pub fn initialize_breaker(
        ctx: Context<InitializeBreaker>,
        params: InitializeBreakerParams,
    ) -> Result<()> {
        instructions::initialize::process_initialize(ctx, params)
    }

    pub fn guarded_withdraw(
        ctx: Context<GuardedWithdraw>,
        amount: u64,
        destination: Pubkey,
    ) -> Result<()> {
        instructions::guarded_withdraw::process_guarded_withdraw(ctx, amount, destination)
    }

    pub fn trip(ctx: Context<Trip>) -> Result<()> {
        instructions::trip_ix::process_trip(ctx)
    }

    pub fn resume(ctx: Context<Resume>) -> Result<()> {
        instructions::resume_ix::process_resume(ctx)
    }

    pub fn propose_config_change(
        ctx: Context<ProposeConfigChange>,
        new_params: ProposedConfigParams,
        requested_delay: i64,
    ) -> Result<()> {
        instructions::propose_config_change::process_propose_config_change(
            ctx,
            new_params,
            requested_delay,
        )
    }

    pub fn execute_config_change(ctx: Context<ExecuteConfigChange>) -> Result<()> {
        instructions::execute_config_change::process_execute_config_change(ctx)
    }

    pub fn cancel_config_change(ctx: Context<CancelConfigChange>) -> Result<()> {
        instructions::cancel_config_change::process_cancel_config_change(ctx)
    }

    pub fn emergency_route_to_safe(
        ctx: Context<EmergencyRouteToSafe>,
        amount: u64,
    ) -> Result<()> {
        instructions::emergency_route_to_safe::process_emergency_route_to_safe(ctx, amount)
    }
}
