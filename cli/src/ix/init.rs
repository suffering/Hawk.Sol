use anchor_lang::{InstructionData, ToAccountMetas};
use circuit_breaker::accounts::InitializeBreaker;
use circuit_breaker::instruction::InitializeBreaker as InitializeBreakerIx;
use circuit_breaker::state::{AUTHORITY_SEED, BREAKER_CONFIG_SEED, WINDOW_STATE_SEED};
use circuit_breaker::{InitializeBreakerParams, ID as PROGRAM_ID};
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    system_program,
};

pub struct InitIxAccounts {
    pub payer: Pubkey,
    pub vault_authority: Pubkey,
    pub mint: Pubkey,
    pub vault: Pubkey,
    pub params: InitializeBreakerParams,
}

pub fn build_initialize_ix(accounts: &InitIxAccounts) -> Instruction {
    let (breaker_config, _) =
        Pubkey::find_program_address(&[BREAKER_CONFIG_SEED, accounts.vault.as_ref()], &PROGRAM_ID);
    let (window_state, _) =
        Pubkey::find_program_address(&[WINDOW_STATE_SEED, accounts.vault.as_ref()], &PROGRAM_ID);
    let (breaker_authority, _) =
        Pubkey::find_program_address(&[AUTHORITY_SEED, accounts.vault.as_ref()], &PROGRAM_ID);

    let metas = InitializeBreaker {
        payer: accounts.payer,
        vault_authority: accounts.vault_authority,
        token_mint: accounts.mint,
        vault: accounts.vault,
        breaker_config,
        window_state,
        breaker_authority,
        token_program: spl_token::ID,
        system_program: system_program::ID,
    };

    Instruction {
        program_id: PROGRAM_ID,
        accounts: metas.to_account_metas(None),
        data: InitializeBreakerIx {
            params: accounts.params.clone(),
        }
        .data(),
    }
}
