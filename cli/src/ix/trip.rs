use anchor_lang::{InstructionData, ToAccountMetas};
use circuit_breaker::accounts::Trip;
use circuit_breaker::instruction::Trip as TripIx;
use circuit_breaker::state::BREAKER_CONFIG_SEED;
use circuit_breaker::ID as PROGRAM_ID;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};

pub fn build_trip_ix(vault: &Pubkey, guardian: &Pubkey) -> Instruction {
    let (breaker_config, _) =
        Pubkey::find_program_address(&[BREAKER_CONFIG_SEED, vault.as_ref()], &PROGRAM_ID);

    let accounts = Trip {
        guardian: *guardian,
        breaker_config,
    };

    Instruction {
        program_id: PROGRAM_ID,
        accounts: accounts.to_account_metas(None),
        data: TripIx {}.data(),
    }
}
