// Manual smoke (localnet):
//   solana-test-validator  (from $TMP, not /mnt/c on WSL drvfs)
//   anchor deploy
//   solhawk demo
//   solhawk demo --seed 42 --json

mod act_attack;
mod act_normal;
mod json;
mod keypairs;
mod preflight;
mod render;
mod setup;
mod summary;

use anyhow::Result;

use crate::cli::{DemoArgs, GlobalOpts};

pub use json::{build_demo_json, print_demo_json, DemoJson};
pub use keypairs::DemoKeypairs;
pub use summary::{compute_summary, SummaryInput, SummaryOutput};

use act_attack::run_act_attack;
use act_normal::run_act_normal;
use preflight::{connect_rpc, ensure_program_deployed};
use render::DemoUi;
use setup::setup_chain;

pub fn run(global: &GlobalOpts, args: DemoArgs) -> Result<()> {
    let ui = DemoUi::new(global.quiet, args.json, args.mint_decimals);
    let keys = DemoKeypairs::from_seed(args.seed);
    let token_kps = DemoKeypairs::token_accounts_from_seed(args.seed);

    ui.println_preflight(&format!(
        "SolHawk demo (seed={}) — localnet proof of efficacy",
        args.seed
    ));

    let rpc = connect_rpc(&global.rpc_url, args.spawn_validator)?;
    ensure_program_deployed(&rpc)?;

    let chain = setup_chain(&rpc, &keys, &token_kps, &args, &ui)?;

    let act_i = run_act_normal(&rpc, &keys, &chain, &args, &ui)?;
    let act_ii = run_act_attack(&rpc, &keys, &chain, &args, &ui)?;

    let summary_input = SummaryInput {
        vault_initial: chain.vault_initial_balance,
        vault_before_act_ii: act_i.vault_before_act_ii,
        vault_after_act_ii: act_ii.vault_after_act_ii,
        threshold: args.threshold,
        mint_decimals: args.mint_decimals,
        unit_price: args.unit_price,
    };

    ui.println_summary(
        summary_input,
        act_ii.attack_attempts_before_trip,
        act_ii.blocked_attempts_after_trip,
    );

    if args.json {
        let payload = json::build_demo_json(
            args.seed,
            &chain.vault,
            &chain.mint,
            summary_input,
            act_ii.attack_attempts_before_trip,
            act_ii.blocked_attempts_after_trip,
        );
        json::print_demo_json(&payload)?;
    }

    Ok(())
}
