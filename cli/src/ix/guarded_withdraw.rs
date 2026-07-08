use anchor_lang::{InstructionData, ToAccountMetas};
use circuit_breaker::accounts::GuardedWithdraw;
use circuit_breaker::instruction::GuardedWithdraw as GuardedWithdrawIx;
use circuit_breaker::state::{AUTHORITY_SEED, BREAKER_CONFIG_SEED, WINDOW_STATE_SEED};
use circuit_breaker::ID as PROGRAM_ID;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};

pub struct GuardedWithdrawAccounts {
    pub operator: Pubkey,
    pub vault: Pubkey,
    pub destination: Pubkey,
}

/// Operator-signed `guarded_withdraw` — account metas match the integration-test harness.
pub fn build_guarded_withdraw_ix(
    accounts: &GuardedWithdrawAccounts,
    amount: u64,
) -> Instruction {
    let (breaker_config, _) =
        Pubkey::find_program_address(&[BREAKER_CONFIG_SEED, accounts.vault.as_ref()], &PROGRAM_ID);
    let (window_state, _) =
        Pubkey::find_program_address(&[WINDOW_STATE_SEED, accounts.vault.as_ref()], &PROGRAM_ID);
    let (breaker_authority, _) =
        Pubkey::find_program_address(&[AUTHORITY_SEED, accounts.vault.as_ref()], &PROGRAM_ID);

    let metas = GuardedWithdraw {
        operator: accounts.operator,
        breaker_config,
        window_state,
        vault: accounts.vault,
        destination: accounts.destination,
        breaker_authority,
        token_program: spl_token::ID,
    };

    Instruction {
        program_id: PROGRAM_ID,
        accounts: metas.to_account_metas(None),
        data: GuardedWithdrawIx {
            amount,
            destination: accounts.destination,
        }
        .data(),
    }
}
