use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::signature::Signer;

use crate::cli::DemoArgs;
use crate::rpc::{execute_transaction, fetch_status_snapshot};

use super::keypairs::DemoKeypairs;
use super::render::DemoUi;
use super::setup::{build_withdraw_ix, token_balance, DemoChain};

pub struct ActNormalResult {
    pub vault_before_act_ii: u64,
}

pub fn run_act_normal(
    rpc: &RpcClient,
    keys: &DemoKeypairs,
    chain: &DemoChain,
    args: &DemoArgs,
    ui: &DemoUi,
) -> Result<ActNormalResult> {
    ui.println_act_header("Act I — normal operations");

    for i in 0..args.normal_count {
        let ix = build_withdraw_ix(
            keys.operator.pubkey(),
            chain.vault,
            chain.safe_destination,
            args.normal_amount,
        );
        execute_transaction(rpc, &[ix], &[&keys.operator])?;

        let snap = fetch_status_snapshot(rpc, &chain.vault)?;
        ui.println_withdraw_ok(i + 1, args.normal_count, &snap);
    }

    let vault_before_act_ii = token_balance(rpc, &chain.vault)?;
    Ok(ActNormalResult {
        vault_before_act_ii,
    })
}
