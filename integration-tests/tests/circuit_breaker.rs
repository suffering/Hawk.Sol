use circuit_breaker::state::{BreakerState, ThresholdMode};
use circuit_breaker::InitializeBreakerParams;
use circuit_breaker_tests::{BreakerTestContext, TestKeypairs};
use solana_sdk::signature::Signer;

#[test]
fn initialize_sets_breaker_pda_as_vault_authority() {
    let mut ctx = BreakerTestContext::new();
    let keys = TestKeypairs::new();
    ctx.setup_vault(&keys, 1_000_000_000);

    ctx.initialize_breaker(&keys, default_params(&keys));

    let vault = ctx.get_token_account(&ctx.vault);
    let (authority, _) = ctx.derive_authority();
    assert_eq!(vault.owner, authority);
}

#[test]
fn happy_path_small_withdraw_succeeds() {
    let mut ctx = BreakerTestContext::new();
    let keys = TestKeypairs::new();
    ctx.setup_vault(&keys, 1_000_000_000);
    ctx.initialize_breaker(&keys, default_params(&keys));

    ctx.guarded_withdraw(&keys, 100_000).expect("withdraw");

    assert_eq!(ctx.get_token_account(&ctx.vault).amount, 999_900_000);
    assert_eq!(ctx.get_token_account(&ctx.destination).amount, 100_000);
}

#[test]
fn velocity_trip_blocks_subsequent_withdrawals() {
    let mut ctx = BreakerTestContext::new();
    let keys = TestKeypairs::new();
    ctx.setup_vault(&keys, 1_000_000_000);

    let mut params = default_params(&keys);
    params.max_absolute = 300_000;
    params.threshold_mode = ThresholdMode::Absolute;
    ctx.initialize_breaker(&keys, params);

    ctx.guarded_withdraw(&keys, 100_000).unwrap();
    ctx.guarded_withdraw(&keys, 100_000).unwrap();
    ctx.guarded_withdraw(&keys, 100_000).unwrap();

    let trip_result = ctx.guarded_withdraw(&keys, 1);
    assert!(trip_result.is_ok());

    let err2 = ctx.guarded_withdraw(&keys, 1).unwrap_err();
    assert!(format!("{err2:?}").contains("Tripped") || format!("{err2:?}").contains("6000"));

    assert!(matches!(
        ctx.get_breaker_config().state,
        BreakerState::Tripped
    ));
}

#[test]
fn guardian_trip_blocks_withdraw_until_resume() {
    let mut ctx = BreakerTestContext::new();
    let keys = TestKeypairs::new();
    ctx.setup_vault(&keys, 1_000_000_000);

    let mut params = default_params(&keys);
    params.cooldown_seconds = 0;
    params.auto_recover = false;
    ctx.initialize_breaker(&keys, params);

    ctx.trip(&keys).expect("trip");
    assert!(ctx.guarded_withdraw(&keys, 1).is_err());

    ctx.resume(&keys, true).expect("resume");
    ctx.guarded_withdraw(&keys, 50_000).expect("withdraw after resume");
}

#[test]
fn auto_recover_allows_permissionless_resume_after_cooldown() {
    let mut ctx = BreakerTestContext::new();
    let keys = TestKeypairs::new();
    ctx.setup_vault(&keys, 1_000_000_000);

    let mut params = default_params(&keys);
    params.cooldown_seconds = 60;
    params.auto_recover = true;
    ctx.initialize_breaker(&keys, params);

    ctx.trip(&keys).expect("trip");
    ctx.warp_time(120);

    let random_caller = solana_sdk::signer::keypair::Keypair::new();
    ctx.airdrop(&random_caller.pubkey(), 1_000_000_000);
    ctx.resume_with_payer(&random_caller).expect("auto resume");

    ctx.guarded_withdraw(&keys, 25_000).expect("withdraw after auto resume");
}

#[test]
fn compromised_admin_drain_capped_at_threshold() {
    let mut ctx = BreakerTestContext::new();
    let keys = TestKeypairs::new();
    let vault_amount = 1_000_000_000u64;
    ctx.setup_vault(&keys, vault_amount);

    let mut params = default_params(&keys);
    params.max_absolute = 100_000_000;
    params.threshold_mode = ThresholdMode::Absolute;
    ctx.initialize_breaker(&keys, params);

    let mut extracted = 0u64;
    for _ in 0..20 {
        let vault_before = ctx.get_token_account(&ctx.vault).amount;
        let _ = ctx.guarded_withdraw(&keys, 50_000_000);
        let vault_after = ctx.get_token_account(&ctx.vault).amount;
        if vault_before == vault_after {
            break;
        }
        extracted += vault_before - vault_after;
    }

    assert_eq!(extracted, 100_000_000);
    assert_eq!(
        ctx.get_token_account(&ctx.vault).amount,
        vault_amount - extracted
    );
    assert!(matches!(
        ctx.get_breaker_config().state,
        BreakerState::Tripped
    ));
}

fn default_params(keys: &TestKeypairs) -> InitializeBreakerParams {
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
        safe_destination: keys.safe_destination.pubkey(),
    }
}
