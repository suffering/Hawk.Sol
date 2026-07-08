use anyhow::Result;
use circuit_breaker::state::ThresholdMode;
use circuit_breaker::InitializeBreakerParams;
use solana_client::rpc_client::RpcClient;
use solana_sdk::signer::Signer;

use crate::branding::terminal_palette;
use crate::cli::{GlobalOpts, InitArgs, ThresholdModeArg};
use crate::commands::tx_output::print_confirmed_status;
use crate::error::confirm_action;
use crate::ix::{build_initialize_ix, InitIxAccounts};
use crate::rpc::{
    execute_transaction, init_checks::validate_init_preconditions, keypair::load_keypair,
};

pub fn run(global: &GlobalOpts, args: InitArgs) -> Result<()> {
    let safe_destination = args
        .safe_destination
        .ok_or_else(|| anyhow::anyhow!("--safe-destination is required"))?;

    let payer = load_keypair(global.keypair.as_deref())?;
    let vault_authority = load_keypair(args.vault_authority_keypair.as_deref().or(global.keypair.as_deref()))?;

    let guardian = args.guardian.unwrap_or_else(|| payer.pubkey());
    let operator = args.operator.unwrap_or_else(|| payer.pubkey());
    let threshold_mode = threshold_mode_from_arg(args.threshold_mode);

    let params = InitializeBreakerParams {
        guardian,
        operator,
        window_seconds: args.window_seconds,
        threshold_mode,
        max_absolute: args.max_absolute,
        max_bps: args.max_bps,
        cooldown_seconds: args.cooldown_seconds,
        timelock_floor: args.timelock_floor,
        auto_recover: args.auto_recover,
        safe_destination,
    };

    if !confirm_init_summary(&args, &params, &payer, &vault_authority, args.yes)? {
        eprintln!("cancelled");
        return Ok(());
    }

    let rpc = RpcClient::new(global.rpc_url.clone());

    validate_init_preconditions(
        &rpc,
        &args.vault,
        &args.mint,
        &vault_authority.pubkey(),
        args.window_seconds,
        args.max_bps,
    )?;

    let ix = build_initialize_ix(&InitIxAccounts {
        payer: payer.pubkey(),
        vault_authority: vault_authority.pubkey(),
        mint: args.mint,
        vault: args.vault,
        params,
    });

    let signers: Vec<&solana_sdk::signature::Keypair> = if payer.pubkey() == vault_authority.pubkey() {
        vec![&payer]
    } else {
        vec![&payer, &vault_authority]
    };

    let outcome = execute_transaction(&rpc, &[ix], &signers)?;
    let palette = terminal_palette(global.quiet);
    print_confirmed_status(&rpc, &args.vault, &outcome.signature, &palette)
}

fn threshold_mode_from_arg(mode: ThresholdModeArg) -> ThresholdMode {
    match mode {
        ThresholdModeArg::Absolute => ThresholdMode::Absolute,
        ThresholdModeArg::PctOfBalance => ThresholdMode::PctOfBalance,
        ThresholdModeArg::Max => ThresholdMode::Max,
    }
}

fn confirm_init_summary(
    args: &InitArgs,
    params: &InitializeBreakerParams,
    payer: &solana_sdk::signature::Keypair,
    vault_authority: &solana_sdk::signature::Keypair,
    yes: bool,
) -> Result<bool> {
    const IMM: &str = "[IMMUTABLE]";

    println!("Initialize circuit breaker:");
    println!("  Vault              {} {}", args.vault, IMM);
    println!("  Mint               {} {}", args.mint, IMM);
    println!("  Guardian           {} {}", params.guardian, IMM);
    println!("  Operator           {} {}", params.operator, IMM);
    println!(
        "  Safe destination   {} {}",
        params.safe_destination, IMM
    );
    println!(
        "  Window             {}s {}",
        params.window_seconds, IMM
    );
    println!(
        "  Threshold mode     {} {}",
        threshold_mode_label(params.threshold_mode),
        IMM
    );
    println!(
        "  Max absolute       {} {}",
        params.max_absolute, IMM
    );
    println!("  Max bps            {} {}", params.max_bps, IMM);
    println!(
        "  Cooldown           {}s {}",
        params.cooldown_seconds, IMM
    );
    println!(
        "  Timelock floor     {}s {}",
        params.timelock_floor, IMM
    );
    println!(
        "  Auto-recover       {} {}",
        params.auto_recover, IMM
    );
    println!("  Payer              {}", payer.pubkey());
    println!("  Vault authority    {}", vault_authority.pubkey());

    confirm_action("Proceed with initialization?", yes)
}

fn threshold_mode_label(mode: ThresholdMode) -> &'static str {
    match mode {
        ThresholdMode::Absolute => "absolute",
        ThresholdMode::PctOfBalance => "pct_of_balance",
        ThresholdMode::Max => "max",
    }
}
