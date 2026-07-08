use anyhow::{bail, Result};
use circuit_breaker::state::BreakerState;
use solana_client::rpc_client::RpcClient;
use solana_sdk::signature::Signer;

use crate::cli::DemoArgs;
use crate::error::format_breaker_error_code;
use crate::rpc::{fetch_status_snapshot, try_execute_transaction};

use super::keypairs::DemoKeypairs;
use super::render::DemoUi;
use super::setup::{build_withdraw_ix, token_balance, DemoChain};

pub struct ActAttackResult {
    pub vault_after_act_ii: u64,
    pub attack_attempts_before_trip: u32,
    pub blocked_attempts_after_trip: u32,
}

const BLOCKED_AFTER_TRIP: u32 = 2;

pub fn run_act_attack(
    rpc: &RpcClient,
    keys: &DemoKeypairs,
    chain: &DemoChain,
    args: &DemoArgs,
    ui: &DemoUi,
) -> Result<ActAttackResult> {
    ui.println_act_header("Act II — rapid drain attempt");

    let mut attack_attempts_before_trip = 0u32;
    let mut tripped = false;

    while !tripped {
        let ix = build_withdraw_ix(
            keys.operator.pubkey(),
            chain.vault,
            chain.drain_destination,
            args.attack_chunk,
        );

        match try_execute_transaction(rpc, &[ix], &[&keys.operator]) {
            Ok(_) => {
                attack_attempts_before_trip += 1;
                let snap = fetch_status_snapshot(rpc, &chain.vault)?;
                if snap.state == BreakerState::Tripped {
                    tripped = true;
                    let msg = format_breaker_error_code(6002)
                        .unwrap_or("Outflow velocity threshold exceeded; breaker tripped");
                    ui.println_trip_detected(msg, &snap);
                } else if snap.state != BreakerState::Active {
                    bail!("unexpected breaker state during attack: {:?}", snap.state);
                }
            }
            Err(failure) => {
                if failure.breaker_code == Some(6000) {
                    attack_attempts_before_trip += 1;
                    tripped = true;
                    let snap = fetch_status_snapshot(rpc, &chain.vault)?;
                    ui.println_trip_detected(&failure.message, &snap);
                } else {
                    bail!(failure.message);
                }
            }
        }

        if attack_attempts_before_trip > 32 {
            bail!("attack loop exceeded safety limit without detecting trip");
        }
    }

    let mut blocked_attempts_after_trip = 0u32;
    while blocked_attempts_after_trip < BLOCKED_AFTER_TRIP {
        let ix = build_withdraw_ix(
            keys.operator.pubkey(),
            chain.vault,
            chain.drain_destination,
            args.attack_chunk,
        );

        match try_execute_transaction(rpc, &[ix], &[&keys.operator]) {
            Ok(_) => bail!("post-trip withdrawal succeeded unexpectedly"),
            Err(failure) => {
                if failure.breaker_code == Some(6000) {
                    blocked_attempts_after_trip += 1;
                    ui.println_blocked_attempt(blocked_attempts_after_trip, &failure.message);
                } else {
                    bail!(failure.message);
                }
            }
        }
    }

    let vault_after_act_ii = token_balance(rpc, &chain.vault)?;

    Ok(ActAttackResult {
        vault_after_act_ii,
        attack_attempts_before_trip,
        blocked_attempts_after_trip,
    })
}
