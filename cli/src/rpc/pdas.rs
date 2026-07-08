use circuit_breaker::state::{BREAKER_CONFIG_SEED, WINDOW_STATE_SEED};
use solana_sdk::pubkey::Pubkey;

pub fn breaker_pdas(program_id: &Pubkey, vault: &Pubkey) -> (Pubkey, Pubkey) {
    let (breaker_config, _) =
        Pubkey::find_program_address(&[BREAKER_CONFIG_SEED, vault.as_ref()], program_id);
    let (window_state, _) =
        Pubkey::find_program_address(&[WINDOW_STATE_SEED, vault.as_ref()], program_id);
    (breaker_config, window_state)
}
