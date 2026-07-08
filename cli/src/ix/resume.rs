use anchor_lang::{InstructionData, ToAccountMetas};
use circuit_breaker::accounts::Resume;
use circuit_breaker::instruction::Resume as ResumeIx;
use circuit_breaker::state::BREAKER_CONFIG_SEED;
use circuit_breaker::ID as PROGRAM_ID;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};

pub fn build_resume_ix(vault: &Pubkey, payer: &Pubkey) -> Instruction {
    let (breaker_config, _) =
        Pubkey::find_program_address(&[BREAKER_CONFIG_SEED, vault.as_ref()], &PROGRAM_ID);

    let accounts = Resume {
        payer: *payer,
        breaker_config,
    };

    Instruction {
        program_id: PROGRAM_ID,
        accounts: accounts.to_account_metas(None),
        data: ResumeIx {}.data(),
    }
}
