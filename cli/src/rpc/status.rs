use anyhow::{ensure, Context, Result};
use circuit_breaker::ID as PROGRAM_ID;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::sysvar::clock::id as clock_sysvar_id;

use crate::model::{FetchedInputs, StatusSnapshot};

use super::decode::{
    decode_breaker_config, decode_clock_sysvar, decode_token_account_amount, decode_window_state,
};
use super::pdas::breaker_pdas;

/// Single gather + assemble. Fetches breaker config, window state, vault
/// balance, and Clock sysvar in one `get_multiple_accounts` call, then
/// builds a frozen [`StatusSnapshot`].
pub fn fetch_status_snapshot(rpc: &RpcClient, vault: &Pubkey) -> Result<StatusSnapshot> {
    let (breaker_config, window_state) = breaker_pdas(&PROGRAM_ID, vault);

    let keys = [
        breaker_config,
        window_state,
        *vault,
        clock_sysvar_id(),
    ];
    let accounts = rpc
        .get_multiple_accounts(&keys)
        .context("fetch breaker accounts from RPC")?;

    let breaker_data = accounts[0]
        .as_ref()
        .map(|a| a.data.as_slice())
        .ok_or_else(|| {
            anyhow::anyhow!("no breaker initialized for vault {vault}")
        })?;

    let window_data = accounts[1]
        .as_ref()
        .map(|a| a.data.as_slice())
        .ok_or_else(|| {
            anyhow::anyhow!("window state account missing for vault {vault}")
        })?;

    let vault_data = accounts[2]
        .as_ref()
        .map(|a| a.data.as_slice())
        .ok_or_else(|| {
            anyhow::anyhow!("vault token account not found: {vault}")
        })?;

    let clock_data = accounts[3]
        .as_ref()
        .map(|a| a.data.as_slice())
        .ok_or_else(|| anyhow::anyhow!("Clock sysvar unavailable"))?;

    let config = decode_breaker_config(breaker_data)?;
    let window = decode_window_state(window_data)?;
    let vault_balance = decode_token_account_amount(vault_data)?;
    let clock = decode_clock_sysvar(clock_data)?;

    ensure!(
        config.vault == *vault,
        "breaker config vault mismatch: config has {}, requested {vault}",
        config.vault
    );

    Ok(StatusSnapshot::from_fetched(FetchedInputs {
        vault: *vault,
        breaker_config,
        window_state,
        vault_balance,
        unix_timestamp: clock.unix_timestamp,
        config,
        window,
    }))
}
