// Manual smoke (localnet):
//   solana-test-validator  (from $TMP, not /mnt/c on WSL drvfs)
//   anchor deploy
//   solhawk demo
//   solhawk demo --seed 42 --json

use anyhow::{bail, Result};
use solana_client::client_error::{ClientError, ClientErrorKind};
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_request::{RpcError, RpcResponseErrorData};
use solana_sdk::{
    instruction::Instruction,
    signature::{Keypair, Signature},
    signer::Signer,
    transaction::Transaction,
};

use crate::error::{
    extract_breaker_error_code, format_client_error, format_simulation_error,
};

pub struct TxOutcome {
    pub signature: Signature,
}

/// Structured transaction failure for orchestration (e.g. demo Act II).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxFailure {
    pub message: String,
    pub breaker_code: Option<u32>,
}

impl TxFailure {
    pub fn from_transaction_error(err: &solana_sdk::transaction::TransactionError) -> Self {
        Self {
            message: format_simulation_error(err),
            breaker_code: extract_breaker_error_code(err),
        }
    }

    pub fn from_client_error(err: &ClientError) -> Self {
        let breaker_code = match err.kind() {
            ClientErrorKind::TransactionError(te) => extract_breaker_error_code(te),
            ClientErrorKind::RpcError(RpcError::RpcResponseError { data, .. }) => {
                if let RpcResponseErrorData::SendTransactionPreflightFailure(preflight) = data {
                    preflight
                        .err
                        .as_ref()
                        .and_then(|te| extract_breaker_error_code(te))
                } else {
                    None
                }
            }
            _ => None,
        };
        Self {
            message: format_client_error(err),
            breaker_code,
        }
    }
}

/// Build, sign once, simulate, then send the same transaction object.
pub fn execute_transaction(
    rpc: &RpcClient,
    instructions: &[Instruction],
    signers: &[&Keypair],
) -> Result<TxOutcome> {
    try_execute_transaction(rpc, instructions, signers).map_err(|f| anyhow::anyhow!(f.message))
}

/// Same pipeline as [`execute_transaction`], returning structured failure.
pub fn try_execute_transaction(
    rpc: &RpcClient,
    instructions: &[Instruction],
    signers: &[&Keypair],
) -> std::result::Result<TxOutcome, TxFailure> {
    ensure_signers(signers).map_err(|e| TxFailure {
        message: e.to_string(),
        breaker_code: None,
    })?;

    let blockhash = rpc
        .get_latest_blockhash()
        .map_err(|e| TxFailure {
            message: format!("fetch latest blockhash: {e}"),
            breaker_code: None,
        })?;

    let tx = Transaction::new_signed_with_payer(
        instructions,
        Some(&signers[0].pubkey()),
        signers,
        blockhash,
    );

    let sim = rpc.simulate_transaction(&tx).map_err(|e| TxFailure {
        message: format!("simulate transaction: {e}"),
        breaker_code: None,
    })?;

    if let Some(err) = sim.value.err {
        return Err(TxFailure::from_transaction_error(&err));
    }

    let signature = rpc
        .send_and_confirm_transaction(&tx)
        .map_err(|err| TxFailure::from_client_error(&err))?;

    Ok(TxOutcome { signature })
}

fn ensure_signers(signers: &[&Keypair]) -> Result<()> {
    if signers.is_empty() {
        bail!("transaction requires at least one signer");
    }
    Ok(())
}
