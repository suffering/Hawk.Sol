pub mod error;
pub mod events;
pub mod instructions;
pub mod state;
pub mod window;

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::error::BreakerError;
use crate::state::{
    BreakerConfig, WindowState, AUTHORITY_SEED, BREAKER_CONFIG_SEED, WINDOW_STATE_SEED,
};

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
}
