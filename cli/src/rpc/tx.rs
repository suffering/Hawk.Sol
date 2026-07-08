// Manual smoke (localnet):
//   solhawk init <VAULT> --mint <MINT> --safe-destination <DEST> --yes
//   solhawk status <VAULT>
//   solhawk trip <VAULT> --yes
//   solhawk resume <VAULT> --yes

use anyhow::{bail, Context, Result};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::Instruction,
    signature::{Keypair, Signature},
    signer::Signer,
    transaction::Transaction,
};

use crate::error::{format_client_error, format_simulation_error};

pub struct TxOutcome {
    pub signature: Signature,
}

/// Build, sign once, simulate, then send the same transaction object.
pub fn execute_transaction(
    rpc: &RpcClient,
    instructions: &[Instruction],
    signers: &[&Keypair],
) -> Result<TxOutcome> {
    ensure_signers(signers)?;

    let blockhash = rpc
        .get_latest_blockhash()
        .context("fetch latest blockhash")?;

    let tx = Transaction::new_signed_with_payer(
        instructions,
        Some(&signers[0].pubkey()),
        signers,
        blockhash,
    );

    let sim = rpc
        .simulate_transaction(&tx)
        .context("simulate transaction")?;

    if let Some(err) = sim.value.err {
        bail!("{}", format_simulation_error(&err));
    }

    let signature = rpc
        .send_and_confirm_transaction(&tx)
        .map_err(|e| anyhow::anyhow!("{}", format_client_error(&e)))?;

    Ok(TxOutcome { signature })
}

fn ensure_signers(signers: &[&Keypair]) -> Result<()> {
    if signers.is_empty() {
        bail!("transaction requires at least one signer");
    }
    Ok(())
}
