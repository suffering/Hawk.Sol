use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

use crate::config::PROGRAM_ID;

pub fn connect_rpc(rpc_url: &str, spawn_validator: bool) -> Result<RpcClient> {
    let rpc = RpcClient::new(rpc_url.to_string());
    if rpc.get_health().is_ok() {
        return Ok(rpc);
    }

    if !spawn_validator {
        bail!(
            "RPC unreachable at {rpc_url}; start `solana-test-validator` or pass --spawn-validator"
        );
    }

    spawn_test_validator().context("spawn solana-test-validator")?;
    wait_for_rpc(&rpc, Duration::from_secs(90))?;
    Ok(rpc)
}

pub fn ensure_program_deployed(rpc: &RpcClient) -> Result<()> {
    let program_id: Pubkey = PROGRAM_ID.parse().context("parse PROGRAM_ID")?;
    let account = rpc
        .get_account(&program_id)
        .context("fetch circuit_breaker program account")?;

    if !account.executable {
        bail!(
            "circuit_breaker program is not deployed at {program_id}; run `anchor deploy` from Hawk.Sol"
        );
    }

    Ok(())
}

fn spawn_test_validator() -> Result<()> {
    let ledger_dir = std::env::temp_dir().join("solhawk-demo-validator");
    let _ = std::fs::remove_dir_all(&ledger_dir);

    Command::new("solana-test-validator")
        .arg("--ledger")
        .arg(&ledger_dir)
        .arg("--quiet")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("failed to spawn solana-test-validator (is it on PATH?)")?;

    Ok(())
}

fn wait_for_rpc(rpc: &RpcClient, timeout: Duration) -> Result<()> {
    let start = Instant::now();
    while start.elapsed() < timeout {
        if rpc.get_health().is_ok() {
            thread::sleep(Duration::from_millis(500));
            return Ok(());
        }
        thread::sleep(Duration::from_millis(250));
    }
    bail!("timed out waiting for validator RPC to become healthy");
}
