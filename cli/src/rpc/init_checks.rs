use anyhow::{bail, ensure, Context, Result};
use circuit_breaker::state::BUCKET_COUNT;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use spl_token::solana_program::program_pack::Pack;
use spl_token::state::Account as TokenAccount;

use crate::rpc::pdas::breaker_pdas;
use circuit_breaker::ID as PROGRAM_ID;

pub fn validate_init_preconditions(
    rpc: &RpcClient,
    vault: &Pubkey,
    mint: &Pubkey,
    vault_authority: &Pubkey,
    window_seconds: i64,
    max_bps: u16,
) -> Result<()> {
    let (breaker_config, _) = breaker_pdas(&PROGRAM_ID, vault);

    let accounts = rpc
        .get_multiple_accounts(&[breaker_config, *vault])
        .context("fetch vault and breaker accounts")?;

    if accounts[0].is_some() {
        bail!("breaker already initialized for vault {vault}");
    }

    let vault_account = accounts[1]
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("vault token account not found: {vault}"))?;

    let token = TokenAccount::unpack(&vault_account.data).context("unpack vault token account")?;

    ensure!(
        token.mint == *mint,
        "vault mint {} does not match --mint {}",
        token.mint,
        mint
    );
    ensure!(
        token.owner == *vault_authority,
        "vault owner {} does not match vault authority keypair {}",
        token.owner,
        vault_authority
    );

    if window_seconds <= 0 || window_seconds % BUCKET_COUNT as i64 != 0 {
        bail!(
            "window_seconds must be positive and divisible by {}",
            BUCKET_COUNT
        );
    }

    if max_bps > 10_000 {
        bail!("max_bps must be <= 10_000");
    }

    Ok(())
}
