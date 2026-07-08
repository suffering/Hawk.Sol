use circuit_breaker::state::{BreakerState, ThresholdMode};
use circuit_breaker::{InitializeBreakerParams, ProposedConfigParams};
use circuit_breaker_tests::{BreakerTestContext, TestKeypairs};
use solana_sdk::{pubkey::Pubkey, signature::Signer};

fn default_params(keys: &TestKeypairs, safe_destination: Pubkey) -> InitializeBreakerParams {
    InitializeBreakerParams {
        guardian: keys.guardian.pubkey(),
        operator: keys.operator.pubkey(),
        window_seconds: 3600,
        threshold_mode: ThresholdMode::Max,
        max_absolute: 500_000_000,
        max_bps: 5000,
        cooldown_seconds: 300,
        timelock_floor: 86400,
        auto_recover: false,
        safe_destination,
    }
}

fn default_proposed(keys: &TestKeypairs) -> ProposedConfigParams {
    ProposedConfigParams {
        guardian: keys.guardian.pubkey(),
        operator: keys.operator.pubkey(),
        window_seconds: 3600,
        threshold_mode: ThresholdMode::Max,
        max_absolute: 500_000_000,
        max_bps: 5000,
        cooldown_seconds: 300,
        auto_recover: false,
    }
}

fn setup_initialized(ctx: &mut BreakerTestContext, keys: &TestKeypairs) {
    ctx.setup_vault(keys, 1_000_000_000);
    let safe_destination = ctx.destination;
    ctx.initialize_breaker(keys, default_params(keys, safe_destination));
}

#[test]
fn propose_execute_after_timelock_succeeds_execute_early_fails() {
    let mut ctx = BreakerTestContext::new();
    let keys = TestKeypairs::new();
    setup_initialized(&mut ctx, &keys);

    let mut proposed = default_proposed(&keys);
    proposed.max_absolute = 200_000_000;

    ctx.propose_config_change(&keys, proposed, 86_400)
        .expect("propose");

    let pending = ctx.get_pending_config();
    assert!(pending.execute_after_ts > pending.proposed_at);

    let early = ctx.execute_config_change(&keys).unwrap_err();
    assert!(
        early.contains("6013") || early.contains("TimelockNotElapsed"),
        "early execute: {early}"
    );

    ctx.warp_time(86_401);
    ctx.execute_config_change(&keys).expect("execute after timelock");

    assert_eq!(ctx.get_breaker_config().max_absolute, 200_000_000);
    assert!(!ctx.pending_config_exists());
}

#[test]
fn requested_delay_below_floor_clamps_to_timelock_floor() {
    let mut ctx = BreakerTestContext::new();
    let keys = TestKeypairs::new();
    setup_initialized(&mut ctx, &keys);

    let floor = ctx.get_breaker_config().timelock_floor;
    ctx.propose_config_change(&keys, default_proposed(&keys), 60)
        .expect("propose");

    let pending = ctx.get_pending_config();
    assert_eq!(pending.effective_delay_seconds, floor);
    assert_eq!(pending.execute_after_ts, pending.proposed_at + floor);
}

#[test]
fn operator_cannot_propose_execute_cancel_or_emergency_route() {
    let mut ctx = BreakerTestContext::new();
    let keys = TestKeypairs::new();
    setup_initialized(&mut ctx, &keys);

    use anchor_lang::{InstructionData, ToAccountMetas};
    use circuit_breaker::accounts::{
        CancelConfigChange, EmergencyRouteToSafe, ExecuteConfigChange, ProposeConfigChange,
    };
    use circuit_breaker::instruction::{
        CancelConfigChange as CancelConfigChangeIx, EmergencyRouteToSafe as EmergencyRouteToSafeIx,
        ExecuteConfigChange as ExecuteConfigChangeIx, ProposeConfigChange as ProposeConfigChangeIx,
    };
    use circuit_breaker::ID as PROGRAM_ID;
    use solana_sdk::{system_program, transaction::Transaction};

    let (breaker_config, _) = ctx.derive_breaker_config();
    let (pending_config, _) = ctx.derive_pending_config();

    let op_propose_ix = solana_sdk::instruction::Instruction {
        program_id: PROGRAM_ID,
        accounts: ProposeConfigChange {
            guardian: keys.operator.pubkey(),
            payer: keys.payer.pubkey(),
            breaker_config,
            pending_config,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: ProposeConfigChangeIx {
            new_params: default_proposed(&keys),
            requested_delay: 86_400,
        }
        .data(),
    };
    let op_propose = format!(
        "{:?}",
        ctx.svm.send_transaction(Transaction::new_signed_with_payer(
            &[op_propose_ix],
            Some(&keys.payer.pubkey()),
            &[&keys.payer, &keys.operator],
            ctx.svm.latest_blockhash(),
        ))
        .unwrap_err()
    );
    assert!(
        op_propose.contains("6014") || op_propose.contains("UnauthorizedGuardian"),
        "{op_propose}"
    );

    ctx.propose_config_change(&keys, default_proposed(&keys), 86_400)
        .expect("guardian propose");

    let (breaker_config, _) = ctx.derive_breaker_config();
    let (window_state, _) = ctx.derive_window_state();
    let (pending_config, _) = ctx.derive_pending_config();

    let exec_ix = solana_sdk::instruction::Instruction {
        program_id: PROGRAM_ID,
        accounts: ExecuteConfigChange {
            guardian: keys.operator.pubkey(),
            payer: keys.payer.pubkey(),
            breaker_config,
            window_state,
            pending_config,
        }
        .to_account_metas(None),
        data: ExecuteConfigChangeIx {}.data(),
    };
    let exec_err = format!(
        "{:?}",
        ctx.svm.send_transaction(Transaction::new_signed_with_payer(
            &[exec_ix],
            Some(&keys.payer.pubkey()),
            &[&keys.payer, &keys.operator],
            ctx.svm.latest_blockhash(),
        ))
        .unwrap_err()
    );
    assert!(
        exec_err.contains("6014") || exec_err.contains("UnauthorizedGuardian"),
        "{exec_err}"
    );

    let cancel_ix = solana_sdk::instruction::Instruction {
        program_id: PROGRAM_ID,
        accounts: CancelConfigChange {
            guardian: keys.operator.pubkey(),
            payer: keys.payer.pubkey(),
            breaker_config,
            pending_config,
        }
        .to_account_metas(None),
        data: CancelConfigChangeIx {}.data(),
    };
    let cancel_err = format!(
        "{:?}",
        ctx.svm.send_transaction(Transaction::new_signed_with_payer(
            &[cancel_ix],
            Some(&keys.payer.pubkey()),
            &[&keys.payer, &keys.operator],
            ctx.svm.latest_blockhash(),
        ))
        .unwrap_err()
    );
    assert!(
        cancel_err.contains("6014") || cancel_err.contains("UnauthorizedGuardian"),
        "{cancel_err}"
    );

    ctx.trip(&keys).expect("trip");
    let (breaker_config, _) = ctx.derive_breaker_config();
    let (breaker_authority, _) = ctx.derive_authority();
    let route_ix = solana_sdk::instruction::Instruction {
        program_id: PROGRAM_ID,
        accounts: EmergencyRouteToSafe {
            guardian: keys.operator.pubkey(),
            breaker_config,
            vault: ctx.vault,
            safe_destination: ctx.destination,
            breaker_authority,
            token_program: spl_token::ID,
        }
        .to_account_metas(None),
        data: EmergencyRouteToSafeIx { amount: 1 }.data(),
    };
    let route_err = format!(
        "{:?}",
        ctx.svm.send_transaction(Transaction::new_signed_with_payer(
            &[route_ix],
            Some(&keys.operator.pubkey()),
            &[&keys.operator],
            ctx.svm.latest_blockhash(),
        ))
        .unwrap_err()
    );
    assert!(
        route_err.contains("6014") || route_err.contains("UnauthorizedGuardian"),
        "{route_err}"
    );
}

#[test]
fn emergency_route_tripped_only_to_safe_destination() {
    let mut ctx = BreakerTestContext::new();
    let keys = TestKeypairs::new();
    setup_initialized(&mut ctx, &keys);

    let active_err = ctx
        .emergency_route_to_safe(&keys, 1_000_000, ctx.destination)
        .unwrap_err();
    assert!(
        active_err.contains("6016") || active_err.contains("EmergencyRouteRequiresTripped"),
        "active route should fail: {active_err}"
    );

    ctx.trip(&keys).expect("trip");
    let vault_before = ctx.get_token_account(&ctx.vault).amount;
    ctx.emergency_route_to_safe(&keys, 1_000_000, ctx.destination)
        .expect("route while tripped");
    assert_eq!(ctx.get_token_account(&ctx.vault).amount, vault_before - 1_000_000);

    let alt = ctx.create_alt_destination(&keys, &keys.operator.pubkey());
    let wrong_dest_err = ctx
        .emergency_route_to_safe(&keys, 1, alt)
        .unwrap_err();
    assert!(
        wrong_dest_err.contains("6008") || wrong_dest_err.contains("DestinationMismatch"),
        "{wrong_dest_err}"
    );
}

#[test]
fn second_propose_fails_cancel_then_repropose_succeeds() {
    let mut ctx = BreakerTestContext::new();
    let keys = TestKeypairs::new();
    setup_initialized(&mut ctx, &keys);

    ctx.propose_config_change(&keys, default_proposed(&keys), 86_400)
        .expect("first propose");

    let dup = ctx
        .propose_config_change(&keys, default_proposed(&keys), 86_400)
        .unwrap_err();
    assert!(
        dup.contains("6012")
            || dup.contains("PendingConfigExists")
            || dup.contains("already in use"),
        "{dup}"
    );

    ctx.cancel_config_change(&keys).expect("cancel");
    assert!(!ctx.pending_config_exists());

    ctx.propose_config_change(&keys, default_proposed(&keys), 86_400)
        .expect("re-propose");
    assert!(ctx.pending_config_exists());
}

#[test]
fn executed_threshold_change_enforced_on_next_withdraw() {
    let mut ctx = BreakerTestContext::new();
    let keys = TestKeypairs::new();
    setup_initialized(&mut ctx, &keys);

    let mut proposed = default_proposed(&keys);
    proposed.max_absolute = 100_000;
    proposed.threshold_mode = ThresholdMode::Absolute;

    ctx.propose_config_change(&keys, proposed, 86_400)
        .expect("propose");
    ctx.warp_time(86_401);
    ctx.execute_config_change(&keys).expect("execute");

    ctx.guarded_withdraw(&keys, 50_000).expect("under cap");
    ctx.guarded_withdraw(&keys, 50_000).expect("at cap");
    let trip = ctx.guarded_withdraw(&keys, 1);
    assert!(trip.is_ok());
    assert_eq!(ctx.get_breaker_config().state, BreakerState::Tripped);
}

#[test]
fn compromised_operator_cannot_propose_while_tripped() {
    let mut ctx = BreakerTestContext::new();
    let keys = TestKeypairs::new();
    setup_initialized(&mut ctx, &keys);

    ctx.trip(&keys).expect("trip");

    let (breaker_config, _) = ctx.derive_breaker_config();
    let (pending_config, _) = ctx.derive_pending_config();
    use anchor_lang::{InstructionData, ToAccountMetas};
    use circuit_breaker::accounts::ProposeConfigChange;
    use circuit_breaker::instruction::ProposeConfigChange as ProposeConfigChangeIx;
    use circuit_breaker::ID as PROGRAM_ID;
    use solana_sdk::{system_program, transaction::Transaction};

    let accounts = ProposeConfigChange {
        guardian: keys.operator.pubkey(),
        payer: keys.payer.pubkey(),
        breaker_config,
        pending_config,
        system_program: system_program::ID,
    };
    let ix = solana_sdk::instruction::Instruction {
        program_id: PROGRAM_ID,
        accounts: accounts.to_account_metas(None),
        data: ProposeConfigChangeIx {
            new_params: default_proposed(&keys),
            requested_delay: 86_400,
        }
        .data(),
    };
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&keys.payer.pubkey()),
        &[&keys.payer, &keys.operator],
        ctx.svm.latest_blockhash(),
    );
    let err = format!("{:?}", ctx.svm.send_transaction(tx).unwrap_err());
    assert!(
        err.contains("6014") || err.contains("UnauthorizedGuardian"),
        "{err}"
    );
    assert_eq!(ctx.get_breaker_config().state, BreakerState::Tripped);
    assert!(!ctx.pending_config_exists());
}

#[test]
fn invalid_proposed_params_rejected_at_propose() {
    let mut ctx = BreakerTestContext::new();
    let keys = TestKeypairs::new();
    setup_initialized(&mut ctx, &keys);

    let mut bad_window = default_proposed(&keys);
    bad_window.window_seconds = 3601;
    let err = ctx
        .propose_config_change(&keys, bad_window, 86_400)
        .unwrap_err();
    assert!(
        err.contains("6006") || err.contains("InvalidWindowSeconds"),
        "{err}"
    );

    let mut bad_bps = default_proposed(&keys);
    bad_bps.max_bps = 10_001;
    let err = ctx
        .propose_config_change(&keys, bad_bps, 86_400)
        .unwrap_err();
    assert!(
        err.contains("6007") || err.contains("InvalidMaxBps"),
        "{err}"
    );

    let err = ctx.propose_config_change(&keys, default_proposed(&keys), -1);
    assert!(err.is_err());
    let err = err.unwrap_err();
    assert!(
        err.contains("6015") || err.contains("InvalidRequestedDelay"),
        "{err}"
    );
}

#[test]
fn execute_window_seconds_change_resets_window_history() {
    let mut ctx = BreakerTestContext::new();
    let keys = TestKeypairs::new();
    setup_initialized(&mut ctx, &keys);

    ctx.guarded_withdraw(&keys, 100_000_000).expect("withdraw");
    let window_before = ctx.get_window_state();
    let sum_before: u64 = window_before.buckets.iter().sum();
    assert!(sum_before > 0);

    let mut proposed = default_proposed(&keys);
    proposed.window_seconds = 7200;

    ctx.propose_config_change(&keys, proposed, 86_400)
        .expect("propose");
    ctx.warp_time(86_401);
    ctx.execute_config_change(&keys).expect("execute");

    let window_after = ctx.get_window_state();
    let sum_after: u64 = window_after.buckets.iter().sum();
    assert_eq!(sum_after, 0, "window history must reset on window_seconds change");
    assert_eq!(ctx.get_breaker_config().window_seconds, 7200);
}
