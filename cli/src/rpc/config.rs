use anyhow::{Context, Result};
use circuit_breaker::state::BreakerConfig;
use circuit_breaker::ID as PROGRAM_ID;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

use crate::rpc::decode::decode_breaker_config;
use crate::rpc::pdas::breaker_pdas;

pub fn fetch_breaker_config(rpc: &RpcClient, vault: &Pubkey) -> Result<BreakerConfig> {
    let (breaker_config, _) = breaker_pdas(&PROGRAM_ID, vault);
    let accounts = rpc
        .get_multiple_accounts(&[breaker_config])
        .context("fetch breaker config")?;

    let account = accounts[0]
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("no breaker initialized for vault {vault}"))?;

    decode_breaker_config(&account.data)
}
