use anyhow::{Context, Result};
use anchor_lang::AccountDeserialize;
use circuit_breaker::state::{BreakerConfig, WindowState};
use solana_sdk::clock::Clock;
use spl_token::solana_program::program_pack::Pack;
use spl_token::state::Account as TokenAccount;

/// Decode the Clock sysvar account data (bincode layout, not Anchor).
pub fn decode_clock_sysvar(data: &[u8]) -> Result<Clock> {
    bincode::deserialize(data).context("decode Clock sysvar (bincode)")
}

pub fn decode_breaker_config(data: &[u8]) -> Result<BreakerConfig> {
    BreakerConfig::try_deserialize(&mut &data[..]).context("deserialize BreakerConfig account")
}

pub fn decode_window_state(data: &[u8]) -> Result<WindowState> {
    WindowState::try_deserialize(&mut &data[..]).context("deserialize WindowState account")
}

pub fn decode_token_account_amount(data: &[u8]) -> Result<u64> {
    let account = TokenAccount::unpack(data).context("unpack SPL token account")?;
    Ok(account.amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_sysvar_decodes_via_bincode() {
        let clock = Clock {
            slot: 42,
            epoch_start_timestamp: 1_600_000_000,
            epoch: 3,
            leader_schedule_epoch: 4,
            unix_timestamp: 1_700_000_000,
        };
        let data = bincode::serialize(&clock).expect("serialize clock");
        let decoded = decode_clock_sysvar(&data).expect("decode clock");
        assert_eq!(decoded.unix_timestamp, clock.unix_timestamp);
        assert_eq!(decoded.slot, clock.slot);
    }
}
